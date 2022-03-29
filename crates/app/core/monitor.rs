use crate::{
	alert::{AlertMethod, AlertMetric},
	clock::Clock,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use std::{borrow::BorrowMut, fmt, io, str::FromStr};
use modelfox_id::Id;
use time::OffsetDateTime;

/// A Monitor generates alerts when production data exceeds configured thresholds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Monitor {
	pub cadence: MonitorCadence,
	pub id: Id,
	pub methods: Vec<AlertMethod>,
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: String,
}

impl Monitor {
	pub fn get_thresholds(&self) -> (Option<f32>, Option<f32>) {
		(
			self.threshold.difference_upper,
			self.threshold.difference_lower,
		)
	}

	/// Check if the given timestamp is more than one cadence interval behind the current time
	pub async fn is_overdue(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		clock: &Clock,
	) -> Result<bool> {
		// check when the last alert from this cadence was recorded
		let last_run = {
			let result = sqlx::query(
				"
					select
						last_checked
					from
						monitors
					where
						monitors.id = $1
				",
			)
			.bind(&self.id.to_string())
			.fetch_optional(txn.borrow_mut())
			.await?;
			if let Some(row) = result {
				let timestamp = row.try_get(0);
				if timestamp.is_err() {
					None
				} else {
					Some(time::OffsetDateTime::from_unix_timestamp(
						timestamp.unwrap(),
					)?)
				}
			} else {
				None
			}
		};
		// If there is no previous run, we know we need to run.  Early return
		if last_run.is_none() {
			return Ok(true);
		}
		// Otherwise, unwrap it and calculate the expected next run
		let last_run = last_run.unwrap();
		let next_due = self.cadence.add_to_time(last_run);
		let now = clock.now_utc();
		Ok(now >= next_due)
	}

	pub async fn update_timestamp(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		clock: &Clock,
	) -> Result<()> {
		let now = clock.now_utc().unix_timestamp();
		sqlx::query(
			"
				update
					monitors
				set
					last_checked = $1
				where
					id = $2
			",
		)
		.bind(now)
		.bind(self.id.to_string())
		.execute(txn.borrow_mut())
		.await?;

		Ok(())
	}

	pub fn default_title(&self) -> String {
		format!("{} {}", self.cadence, self.threshold.metric)
	}
}

/// For filtering valid metric options
#[derive(Debug, Clone, Copy)]
pub enum AlertModelType {
	Classifier,
	Regressor,
}

impl From<modelfox_model::ModelInnerReader<'_>> for AlertModelType {
	fn from(mir: modelfox_model::ModelInnerReader) -> Self {
		use modelfox_model::ModelInnerReader::*;
		match mir {
			BinaryClassifier(_) | MulticlassClassifier(_) => AlertModelType::Classifier,
			Regressor(_) => AlertModelType::Regressor,
		}
	}
}

/// Alerts can either be set as absolute values or percentage deviations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum MonitorThresholdMode {
	#[serde(rename = "absolute")]
	Absolute,
	#[serde(rename = "percentage")]
	Percentage,
}

impl fmt::Display for MonitorThresholdMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			MonitorThresholdMode::Absolute => "absolute",
			MonitorThresholdMode::Percentage => "percentage",
		};
		write!(f, "{}", s)
	}
}

impl FromStr for MonitorThresholdMode {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"absolute" => Ok(MonitorThresholdMode::Absolute),
			"percentage" => Ok(MonitorThresholdMode::Percentage),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"unsupported threshold mode",
			)),
		}
	}
}

/// Single alert threshold
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct MonitorThreshold {
	pub metric: AlertMetric,
	pub mode: MonitorThresholdMode,
	pub difference_lower: Option<f32>,
	pub difference_upper: Option<f32>,
}

impl Default for MonitorThreshold {
	fn default() -> Self {
		MonitorThreshold {
			metric: AlertMetric::Accuracy,
			mode: MonitorThresholdMode::Absolute,
			difference_lower: Some(0.1),
			difference_upper: Some(0.1),
		}
	}
}

pub fn extract_threshold_bounds(
	threshold_bounds: (String, String),
) -> Result<(Option<f32>, Option<f32>)> {
	let lower = if !threshold_bounds.0.is_empty() {
		Some(threshold_bounds.0.parse()?)
	} else {
		None
	};
	let upper = if !threshold_bounds.1.is_empty() {
		Some(threshold_bounds.1.parse()?)
	} else {
		None
	};
	Ok((lower, upper))
}

pub fn validate_threshold_bounds(lower: String, upper: String) -> Option<(String, String)> {
	let at_least_one = (!lower.is_empty() && !upper.is_empty())
		|| (lower.is_empty() && !upper.is_empty())
		|| (!lower.is_empty() && upper.is_empty());
	if at_least_one {
		Some((lower, upper))
	} else {
		None
	}
}

// Alert cadence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MonitorCadence {
	#[serde(rename = "testing")]
	Testing = 0,
	#[serde(rename = "hourly")]
	Hourly,
	#[serde(rename = "daily")]
	Daily,
	#[serde(rename = "weekly")]
	Weekly,
	#[serde(rename = "monthly")]
	Monthly,
}

impl MonitorCadence {
	/// Add one period of this cadence to a datetime
	pub fn add_to_time(&self, time: OffsetDateTime) -> OffsetDateTime {
		match &self {
			MonitorCadence::Testing => time.checked_add(time::Duration::SECOND).unwrap(),
			MonitorCadence::Hourly => time.checked_add(time::Duration::HOUR).unwrap(),
			MonitorCadence::Daily => time.checked_add(time::Duration::DAY).unwrap(),
			MonitorCadence::Weekly => time.checked_add(time::Duration::WEEK).unwrap(),
			MonitorCadence::Monthly => {
				let last_run_year = time.year();
				let last_run_month = time.month();
				let next_run_month = last_run_month.next();
				let next_run_year = if last_run_month == time::Month::December {
					last_run_year + 1
				} else {
					last_run_year
				};
				let last_run_day = time.day();
				let days_in_next_run_month =
					time::util::days_in_year_month(next_run_year, next_run_month);

				let next_run_day = if last_run_day + 1 > days_in_next_run_month {
					days_in_next_run_month
				} else {
					last_run_day + 1
				};
				let next_run_date =
					time::Date::from_calendar_date(next_run_year, next_run_month, next_run_day)
						.unwrap();
				time.replace_date(next_run_date)
			}
		}
	}
}

impl Default for MonitorCadence {
	fn default() -> Self {
		MonitorCadence::Hourly
	}
}

impl fmt::Display for MonitorCadence {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			MonitorCadence::Testing => "Testing",
			MonitorCadence::Hourly => "Hourly",
			MonitorCadence::Daily => "Daily",
			MonitorCadence::Weekly => "Weekly",
			MonitorCadence::Monthly => "Monthly",
		};
		write!(f, "{}", s)
	}
}

impl FromStr for MonitorCadence {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"daily" => Ok(MonitorCadence::Daily),
			"hourly" => Ok(MonitorCadence::Hourly),
			"monthly" => Ok(MonitorCadence::Monthly),
			"testing" => Ok(MonitorCadence::Testing),
			"weekly" => Ok(MonitorCadence::Weekly),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Unsupported cadence {}", s),
			)),
		}
	}
}

impl TryFrom<u8> for MonitorCadence {
	type Error = std::io::Error;
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(MonitorCadence::Testing),
			1 => Ok(MonitorCadence::Hourly),
			2 => Ok(MonitorCadence::Daily),
			3 => Ok(MonitorCadence::Weekly),
			4 => Ok(MonitorCadence::Monthly),
			_ => Err(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"Must be integer in 0..=4",
			)),
		}
	}
}

impl From<MonitorCadence> for u8 {
	fn from(cadence: MonitorCadence) -> Self {
		match cadence {
			MonitorCadence::Testing => 0,
			MonitorCadence::Hourly => 1,
			MonitorCadence::Daily => 2,
			MonitorCadence::Weekly => 3,
			MonitorCadence::Monthly => 4,
		}
	}
}

pub async fn create_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor: Monitor,
	model_id: Id,
) -> Result<()> {
	let monitor_json = serde_json::to_string(&monitor)?;
	sqlx::query(
		"
			insert into monitors
				(id, model_id, data, cadence)
			values
				($1, $2, $3, $4)
		",
	)
	.bind(monitor.id.to_string())
	.bind(model_id.to_string())
	.bind(monitor_json)
	.bind(u8::from(monitor.cadence) as i64) // FIXME cadence Encode
	.execute(db)
	.await?;
	Ok(())
}

pub async fn update_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	new_monitor: &Monitor,
	monitor_id: Id,
	clock: &Clock,
) -> Result<()> {
	let monitor_json = serde_json::to_string(new_monitor)?;
	sqlx::query(
		"
			update
				monitors
			set data = $1, date = $2
			where id = $3
		",
	)
	.bind(monitor_json)
	.bind(clock.now_utc().unix_timestamp().to_string())
	.bind(monitor_id.to_string())
	.execute(db)
	.await?;
	Ok(())
}

pub async fn check_for_duplicate_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor: &Monitor,
	model_id: Id,
) -> Result<bool> {
	// Pull all rows with matching model
	// Verify none of them have identical thresholds
	let rows = sqlx::query(
		"
			select
				data
			from
				monitors
			where
				model_id = $1
		",
	)
	.bind(model_id.to_string())
	.fetch_all(db)
	.await?;
	let result = rows
		.iter()
		.map(|row| {
			let monitor_json: String = row.get(0);
			let monitor: Monitor =
				serde_json::from_str(&monitor_json).expect("Could not parse stored alert");
			monitor
		})
		.any(|el| el.cadence == monitor.cadence && el.threshold == monitor.threshold);

	Ok(result)
}

pub async fn get_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor_id: Id,
) -> Result<Monitor> {
	let result = sqlx::query(
		"
			select
				data
			from
				monitors
			where
				id = $1
		",
	)
	.bind(monitor_id.to_string())
	.fetch_one(db)
	.await?;
	let monitor: String = result.get(0);
	let monitor: Monitor = serde_json::from_str(&monitor)?;
	Ok(monitor)
}

pub async fn delete_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor_id: &str,
) -> Result<()> {
	sqlx::query(
		"
			delete from monitors
			where id = $1
		",
	)
	.bind(monitor_id.to_string())
	.execute(db)
	.await?;
	Ok(())
}
