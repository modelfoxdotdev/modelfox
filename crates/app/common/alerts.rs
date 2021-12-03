use crate::Context;
use anyhow::{anyhow, Result};
use lettre::AsyncTransport;
use serde::{Deserialize, Serialize};
use std::fmt;

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
			AlertCadence::Daily => "daily",
			AlertCadence::Hourly => "hourly",
			AlertCadence::Monthly => "monthly",
			AlertCadence::Testing => "testing",
			AlertCadence::Weekly => "weekly",
		};
		write!(f, "{}", s)
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
}

impl AlertMethod {
	pub async fn send_alert(
		&self,
		exceeded_thresholds: &[&AlertResult],
		context: &Context,
	) -> Result<()> {
		match self {
			AlertMethod::Email(address) => {
				let email = lettre::Message::builder()
					.from("Tangram <noreply@tangram.dev>".parse()?)
					.to(address.parse()?)
					.subject("Tangram Metrics Alert")
					.body(format!(
						"Exceeded alert thresholds: {:?}",
						exceeded_thresholds
					))?; // TODO - include heuristics too
				if let Some(smtp_transport) = &context.smtp_transport {
					smtp_transport.send(email).await?;
				} else {
					return Err(anyhow!("No SMTP Transport in context"));
				}
			}
			AlertMethod::Stdout => println!("exceeded thresholds: {:?}", exceeded_thresholds),
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
	#[serde(rename = "root_mean_squared_error")]
	RootMeanSquaredError,
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

/// Single alert threshold
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AlertThreshold {
	pub metric: AlertMetric,
	pub variance: f32,
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
	pub thresholds: Vec<AlertThreshold>,
}

impl AlertHeuristics {
	/// Retrieve the variance tolerance for a given metric, if present
	pub fn get_threshold(&self, metric: AlertMetric) -> Option<f32> {
		for threshold in &self.thresholds {
			if threshold.metric == metric {
				return Some(threshold.variance);
			}
		}
		None
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
	pub alert_methods: Vec<AlertMethod>,
	pub heuristics: AlertHeuristics,
	pub results: Vec<AlertResult>,
}
