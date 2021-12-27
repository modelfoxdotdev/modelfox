use crate::Context;
use anyhow::{anyhow, Result};
use lettre::AsyncTransport;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{fmt, io, str::FromStr};
use tangram_id::Id;

// Task

// Types

/// Alert cadence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AlertCadence {
	#[serde(rename = "daily")]
	Daily,
	#[serde(rename = "hourly")]
	Hourly,
	#[serde(rename = "monthly")]
	Monthly,
	#[serde(rename = "testing")]
	Testing,
	#[serde(rename = "weekly")]
	Weekly,
}

impl AlertCadence {
	pub fn duration(&self) -> tokio::time::Duration {
		match self {
			AlertCadence::Daily => tokio::time::Duration::from_secs(60 * 60 * 24),
			AlertCadence::Hourly => tokio::time::Duration::from_secs(60 * 60),
			AlertCadence::Monthly => tokio::time::Duration::from_secs(60 * 60 * 24 * 31), //FIXME that's not correct
			AlertCadence::Testing => tokio::time::Duration::from_secs(10),
			AlertCadence::Weekly => tokio::time::Duration::from_secs(60 * 60 * 24 * 7),
		}
	}
}

impl Default for AlertCadence {
	fn default() -> Self {
		AlertCadence::Hourly
	}
}

impl fmt::Display for AlertCadence {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertCadence::Daily => "Daily",
			AlertCadence::Hourly => "Hourly",
			AlertCadence::Monthly => "Monthly",
			AlertCadence::Testing => "Testing",
			AlertCadence::Weekly => "Weekly",
		};
		write!(f, "{}", s)
	}
}

impl FromStr for AlertCadence {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"daily" => Ok(AlertCadence::Daily),
			"hourly" => Ok(AlertCadence::Hourly),
			"monthly" => Ok(AlertCadence::Monthly),
			"testing" => Ok(AlertCadence::Testing),
			"weekly" => Ok(AlertCadence::Weekly),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("Unsupported cadence {}", s),
			)),
		}
	}
}

/// The various ways to receive alerts
// FIXME - using tag = type and renaming here causes sqlx to panic!!
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum AlertMethod {
	/// Send an email to the stored address
	Email(String),
	/// Dump the alert to STDOUT - mostly useful for testing
	Stdout,
	/// POST the alert to the given URL as a webhook
	Webhook(String),
}

impl AlertMethod {
	pub async fn send_alert(
		&self,
		exceeded_thresholds: &[&AlertResult],
		context: &Context,
	) -> Result<()> {
		match self {
			AlertMethod::Email(_address) => {
				// TODO re-enable this code!
				/*
				let email = lettre::Message::builder()
					.from("Tangram <noreply@tangram.dev>".parse()?)
					.to(address.parse()?)
					.subject("Tangram Metrics Alert")
					.body(format!(
						"Exceeded alert thresholds: {:?}",
						exceeded_thresholds
					))?;
				if let Some(smtp_transport) = &context.smtp_transport {
					smtp_transport.send(email).await?;
				} else {
					return Err(anyhow!("No SMTP Transport in context"));
				}
				*/
			}
			AlertMethod::Stdout => println!("exceeded thresholds: {:?}", exceeded_thresholds),
			AlertMethod::Webhook(_url) => {
				// Spawn a thread
				// Attempt the POST, record status in DB.
				// If status has failed, attempt again until it succeeds.
			}
		}
		Ok(())
	}
}

/// Statistics that can generate alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AlertMetric {
	#[serde(rename = "accuracy")]
	Accuracy,
	#[serde(rename = "mean_squared_error")]
	MeanSquaredError,
	#[serde(rename = "root_mean_squared_error")]
	RootMeanSquaredError,
}

impl AlertMetric {
	pub fn short_name(&self) -> String {
		match self {
			AlertMetric::Accuracy => "accuracy".to_owned(),
			AlertMetric::MeanSquaredError => "mse".to_owned(),
			AlertMetric::RootMeanSquaredError => "rmse".to_owned(),
		}
	}
}

impl fmt::Display for AlertMetric {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertMetric::Accuracy => "Accuracy",
			AlertMetric::MeanSquaredError => "Mean Squared Error",
			AlertMetric::RootMeanSquaredError => "Root Mean Squared Error",
		};
		write!(f, "{}", s)
	}
}

impl FromStr for AlertMetric {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"accuracy" => Ok(AlertMetric::Accuracy),
			"mse" | "mean_squared_error" => Ok(AlertMetric::MeanSquaredError),
			"rmse" | "root_mean_squared_error" => Ok(AlertMetric::RootMeanSquaredError),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"Unsupported alert metric",
			)),
		}
	}
}

/// For filtering valid metric options
pub enum AlertModelType {
	Classifier,
	Regressor,
}

impl From<tangram_model::ModelInnerReader<'_>> for AlertModelType {
	fn from(mir: tangram_model::ModelInnerReader) -> Self {
		use tangram_model::ModelInnerReader::*;
		match mir {
			BinaryClassifier(_) | MulticlassClassifier(_) => AlertModelType::Classifier,
			Regressor(_) => AlertModelType::Regressor,
		}
	}
}

/// An alert record can be in one of these states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertProgress {
	Completed,
	InProgress,
}

impl fmt::Display for AlertProgress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertProgress::Completed => "COMPLETED",
			AlertProgress::InProgress => "IN PROGRESS",
		};
		write!(f, "{}", s)
	}
}

/// Alerts can either be set as absolute values or percentage deviations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum AlertThresholdMode {
	#[serde(rename = "absolute")]
	Absolute,
	#[serde(rename = "percentage")]
	Percentage,
}

impl fmt::Display for AlertThresholdMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	    let s = match self {
				AlertThresholdMode::Absolute => "absolute",
				AlertThresholdMode::Percentage => "percentage",
			};
			write!(f, "{}", s)
	}
}

impl FromStr for AlertThresholdMode {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"absolute" => Ok(AlertThresholdMode::Absolute),
			"percentage" => Ok(AlertThresholdMode::Percentage),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"unsupported threshold mode",
			)),
		}
	}
}

/// Single alert threshold
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct AlertThreshold {
	pub metric: AlertMetric,
	pub mode: AlertThresholdMode,
	pub variance_lower: Option<f32>,
	pub variance_upper: Option<f32>,
}

impl Default for AlertThreshold {
	fn default() -> Self {
		AlertThreshold {
			metric: AlertMetric::Accuracy,
			mode: AlertThresholdMode::Absolute,
			variance_lower: Some(0.1),
			variance_upper: Some(0.1),
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

/// A result from checking a metric
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AlertResult {
	pub metric: AlertMetric,
	pub observed_value: f32,
	pub observed_variance: f32,
}

impl AlertResult {
	/// Should this result send an alert?
	pub fn exceeds_threshold(&self, tolerance: f32) -> bool {
		self.observed_variance.abs() > tolerance
	}
}

/// Thresholds for generating an Alert
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AlertHeuristics {
	pub cadence: AlertCadence,
	pub methods: Vec<AlertMethod>,
	pub threshold: AlertThreshold,
	pub title: String,
}

impl AlertHeuristics {
	pub fn get_threshold(&self, metric: AlertMetric) -> Option<f32> {
		todo!()
	}
	/// Check if the given timestamp is more than one cadence interval behind the current time
	pub fn is_expired(&self, timestamp: u64) -> bool {
		let now = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
		let offset = now - timestamp;
		offset > self.cadence.duration().as_secs()
	}

	pub fn default_title(&self) -> String {
		format!("{} {}", self.cadence, self.threshold.metric)
	}
}

/// Manager for all enabled alerts
#[derive(Debug, Default)]
pub struct Alerts(Vec<AlertHeuristics>);

impl From<Vec<AlertHeuristics>> for Alerts {
	fn from(v: Vec<AlertHeuristics>) -> Self {
		Self(v)
	}
}

impl Alerts {
	// Retrieve all currently enabled cadences
	pub fn get_cadences(&self) -> Vec<AlertCadence> {
		self.0
			.iter()
			.map(|ah| ah.cadence)
			.collect::<Vec<AlertCadence>>()
	}
	pub fn alerts(&self) -> &[AlertHeuristics] {
		&self.0
	}

	/// Retrieve the heuristics for the given cadence, if present
	pub fn cadence(&self, cadence: AlertCadence) -> Option<&AlertHeuristics> {
		for heuristics in &self.0 {
			if heuristics.cadence == cadence {
				return Some(heuristics);
			}
		}
		None
	}
}

/// Collection for the alert results from a single run
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AlertData {
	pub heuristics: AlertHeuristics,
	pub results: Vec<AlertResult>,
}

// Helpers

// Database interaction

pub async fn get_alert(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	alert_id: &str,
) -> Result<AlertHeuristics> {
	let result = sqlx::query(
		"
			select
				alert
			from
				alert_preferences
			where
				id = $1
		",
	)
	.bind(alert_id.to_owned())
	.fetch_one(db)
	.await?;
	let alert: String = result.get(0);
	let alert: AlertHeuristics = serde_json::from_str(&alert)?;
	Ok(alert)
}

pub async fn get_all_alerts(context: &Context) -> Result<Alerts> {
	let mut db = context.database_pool.begin().await?;
	let rows = sqlx::query(
		"
			select
				alert
			from
				alert_preferences
		",
	)
	.fetch_all(&mut db)
	.await?;
	db.commit().await?;
	let alerts: Vec<AlertHeuristics> = rows
		.iter()
		.map(|row| {
			let alert: String = row.get(0);
			serde_json::from_str(&alert).unwrap()
		})
		.collect();
	Ok(Alerts::from(alerts))
}

/// Get the unix timestamp of the last run with the given cadence/metric combination
pub async fn get_last_alert_time(
	context: &Context,
	cadence: AlertCadence,
	metric: AlertMetric,
) -> Result<Option<u64>> {
	let mut db = context.database_pool.begin().await?;
	let result = sqlx::query(
		"
			select
				MAX(date)
			from
				alerts
			where
				cadence = $1 and
				metric = $2
		",
	)
	.bind(cadence.to_string())
	.bind(metric.to_string())
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	match result {
		Some(r) => {
			let data: String = r.get(0);
			if data.is_empty() {
				return Ok(None);
			}
			let data: u64 = data.parse()?;
			Ok(Some(data))
		}
		None => Ok(None),
	}
}

pub async fn check_for_duplicate_alert(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	alert: &AlertHeuristics,
	model_id: Id,
) -> Result<bool> {
	// Pull all rows with matching metric and cadence
	// Verify none of them have identical thresholds
	let rows = sqlx::query(
		"
			select
				alert
			from
				alert_preferences
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
			let alert_json: String = row.get(0);
			let alert: AlertHeuristics =
				serde_json::from_str(&alert_json).expect("Could not parse stored alert");
			alert
		})
		.fold(false, |acc, el| {
			acc || (el.cadence == alert.cadence && el.threshold == alert.threshold)
		});

	Ok(result)
}

pub async fn create_alert(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	alert: AlertHeuristics,
	model_id: Id,
) -> Result<()> {
	let alert_json = serde_json::to_string(&alert)?;
	sqlx::query(
		"
			insert into alert_preferences
				(id, alert, model_id, last_updated)
			values
				($1, $2, $3, $4)
		",
	)
	.bind(Id::generate().to_string())
	.bind(alert_json)
	.bind(model_id.to_string())
	.bind(time::OffsetDateTime::now_utc().unix_timestamp().to_string())
	.execute(db)
	.await?;
	Ok(())
}

pub async fn delete_alert(db: &mut sqlx::Transaction<'_, sqlx::Any>, alert_id: &str) -> Result<()> {
	sqlx::query(
		"
			delete from alert_preferences
			where id = $1
		",
	)
	.bind(alert_id.to_string())
	.execute(db)
	.await?;
	Ok(())
}

pub async fn update_alert(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	new_alert: &AlertHeuristics,
	alert_id: &str,
) -> Result<()> {
	let alert_json = serde_json::to_string(new_alert)?;
	sqlx::query(
		"
			update
				alert_preferences
			set alert = $1, last_updated = $2
			where id = $3
		",
	)
	.bind(alert_json)
	.bind(time::OffsetDateTime::now_utc().unix_timestamp().to_string())
	.bind(alert_id.to_string())
	.execute(db)
	.await?;
	Ok(())
}

/*
/// Handle a specific alert cadence
async fn handle_alert_cadence(
	alerts: &Alerts,
	alert_methods: &[AlertMethod],
	cadence: AlertCadence,
	context: &Context,
) -> Result<()> {
	let already_handled = check_ongoing(cadence, &context.database_pool).await?;

	if already_handled {
		return Ok(());
	}

	let alert_id = write_alert_start(cadence, &context.database_pool).await?;

	let heuristics = alerts.cadence(cadence).unwrap();
	let results = check_metrics(heuristics, &context.database_pool).await?;
	let exceeded_thresholds: Vec<&AlertResult> = results
		.iter()
		.filter(|r| {
			heuristics
				.get_threshold(r.metric)
				.map(|t| r.exceeds_threshold(t))
				.unwrap_or(false)
		})
		.collect();
	push_alerts(&exceeded_thresholds, alert_methods, context).await?;

	let alert_data = AlertData {
		alert_methods: alert_methods.to_owned(),
		heuristics: heuristics.to_owned(),
		results: results.to_owned(),
	};
	write_alert_end(alert_id, alert_data, &context.database_pool).await?;
	Ok(())
}
*/
