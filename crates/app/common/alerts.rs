use crate::{model::get_model_bytes, Context};
use anyhow::{anyhow, Result};
//use lettre::AsyncTransport;
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
	#[serde(rename = "testing")]
	Testing,
	#[serde(rename = "hourly")]
	Hourly,
	#[serde(rename = "daily")]
	Daily,
	#[serde(rename = "weekly")]
	Weekly,
	#[serde(rename = "monthly")]
	Monthly,
}

// store timestamp, truncate to significant portion.

impl AlertCadence {
	pub fn duration(&self) -> tokio::time::Duration {
		// This is really not a duration, it's looking for the time until the next interval?
		// think about this more
		todo!()
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
			AlertCadence::Testing => "Testing",
			AlertCadence::Hourly => "Hourly",
			AlertCadence::Daily => "Daily",
			AlertCadence::Weekly => "Weekly",
			AlertCadence::Monthly => "Monthly",
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
		_context: &Context,
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

	/// Check if the given AlertModelType is applicable to this AlertMetric
	pub fn validate(&self, model_type: AlertModelType) -> bool {
		match self {
			AlertMetric::Accuracy => matches!(model_type, AlertModelType::Classifier),
			AlertMetric::MeanSquaredError | &AlertMetric::RootMeanSquaredError => {
				matches!(model_type, AlertModelType::Regressor)
			}
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
#[derive(Debug, Clone, Copy)]
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
	pub variance_lower: Option<f32>,
	pub variance_upper: Option<f32>,
}

impl Default for MonitorThreshold {
	fn default() -> Self {
		MonitorThreshold {
			metric: AlertMetric::Accuracy,
			mode: MonitorThresholdMode::Absolute,
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

// FIXME - Monitor.  Monitor produces alerts
/// Thresholds for generating an Alert
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Monitor {
	pub cadence: AlertCadence,
	pub id: Id,
	pub methods: Vec<AlertMethod>,
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: String,
}

impl Monitor {
	pub fn get_thresholds(&self) -> (Option<f32>, Option<f32>) {
		(self.threshold.variance_upper, self.threshold.variance_lower)
	}
	/// Check if the given timestamp is more than one cadence interval behind the current time
	// FIXME - this is not correct.  Check when the last one was, when the next one should be, etc.
	// For now, just returns true all the time.
	pub fn is_overdue(&self) -> bool {
		//let now = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
		//let offset = now - timestamp;
		//offset > self.cadence.duration().as_secs()
		true
	}

	pub fn default_title(&self) -> String {
		format!("{} {}", self.cadence, self.threshold.metric)
	}
}

/// Manager for all enabled alerts
#[derive(Debug, Default)]
pub struct Alerts(Vec<Monitor>);

impl From<Vec<Monitor>> for Alerts {
	fn from(v: Vec<Monitor>) -> Self {
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
	pub fn alerts(&self) -> &[Monitor] {
		&self.0
	}

	/// Retrieve the heuristics for the given cadence, if present
	pub fn cadence(&self, cadence: AlertCadence) -> Option<&Monitor> {
		for heuristics in &self.0 {
			if heuristics.cadence == cadence {
				return Some(heuristics);
			}
		}
		None
	}
}

/// Collection for the alert results from a single run
#[derive(Debug, Deserialize, Serialize)]
pub struct AlertData {
	pub preference: Monitor,
	pub results: Vec<AlertResult>,
}

// Helpers

// Database interaction

pub async fn get_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor_id: &str,
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
	.bind(monitor_id.to_owned())
	.fetch_one(db)
	.await?;
	let monitor: String = result.get(0);
	let monitor: Monitor = serde_json::from_str(&monitor)?;
	Ok(monitor)
}

pub async fn get_overdue_monitors(context: &Context) -> Result<Alerts> {
	let mut db = context.database_pool.begin().await?;
	let rows = sqlx::query(
		"
			select
				data
			from
				monitors
		",
	)
	.fetch_all(&mut db)
	.await?;
	db.commit().await?;
	let monitors: Vec<Monitor> = rows
		.iter()
		.map(|row| {
			let monitor: String = row.get(0);
			serde_json::from_str(&monitor).unwrap()
		})
		.filter(|monitor: &Monitor| monitor.is_overdue())
		.collect();
	Ok(Alerts::from(monitors))
}

pub async fn check_for_duplicate_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	monitor: &Monitor,
	model_id: Id,
) -> Result<bool> {
	// Pull all rows with matching metric and cadence
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
		.fold(false, |acc, el| {
			acc || (el.cadence == monitor.cadence && el.threshold == monitor.threshold)
		});

	Ok(result)
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
				(id, model_id, data, cadence, date)
			values
				($1, $2, $3, $4, $5)
		",
	)
	.bind(Id::generate().to_string())
	.bind(model_id.to_string())
	.bind(monitor_json)
	.bind(monitor.cadence.to_string()) // FIXME cadence Encode
	.bind(time::OffsetDateTime::now_utc().unix_timestamp().to_string())
	.execute(db)
	.await?;
	Ok(())
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

pub async fn update_monitor(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	new_monitor: &Monitor,
	monitor_id: &str,
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
	.bind(time::OffsetDateTime::now_utc().unix_timestamp().to_string())
	.bind(monitor_id.to_string())
	.execute(db)
	.await?;
	Ok(())
}

/// Read the model, find the training metric value for the given AlertMetric
pub async fn find_current_data(
	metric: AlertMetric,
	model_id: Id,
	context: &Context,
) -> Result<f32> {
	// Grab the model from the DB
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	// Determine model type
	let model_inner = model.inner();
	let model_type = AlertModelType::from(model_inner);
	if !metric.validate(model_type) {
		return Err(anyhow!(
			"Invalid metric {} for model type {:?}",
			metric,
			model_type
		));
	}

	// match on model_inner
	let result = match metric {
		AlertMetric::Accuracy => {
			// We know we have a classifier, need to get the accuracy from either type
			match model_inner {
				tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
					binary_classifier
						.read()
						.test_metrics()
						.default_threshold() // TODO ask if this is correct?
						.accuracy()
				}
				tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
					multiclass_classifier.read().test_metrics().accuracy()
				}
				_ => unreachable!(),
			}
		}
		AlertMetric::MeanSquaredError => {
			// we know we have a regressor, just read it
			match model_inner {
				tangram_model::ModelInnerReader::Regressor(regressor) => {
					regressor.read().test_metrics().mse()
				}
				_ => unreachable!(),
			}
		}
		AlertMetric::RootMeanSquaredError => {
			// we know we have a regressor, just read it
			match model_inner {
				tangram_model::ModelInnerReader::Regressor(regressor) => {
					regressor.read().test_metrics().rmse()
				}
				_ => unreachable!(),
			}
		}
	};
	Ok(result)
}
