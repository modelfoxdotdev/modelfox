use crate::{
	alert_sender::create_alert_send,
	monitor::{AlertModelType, Monitor},
	App, AppState,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use std::{borrow::BorrowMut, fmt, io, str::FromStr};
use tangram_id::Id;
use time::{macros::format_description, OffsetDateTime};
use url::Url;

/// Collection for the alert results from a single run
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Alert {
	pub id: Id,
	pub monitor: Monitor,
	pub result: AlertResult,
	pub timestamp: i64,
}

impl Alert {
	pub fn cadence_and_metric(&self) -> String {
		let cadence = self.monitor.cadence;
		let metric = self.metric();
		format!("{cadence} {metric}")
	}

	pub fn formated_time_range(&self) -> String {
		let (begin_time, end_time) = self.time_range().unwrap();
		let dt_format = format_description!("[year]-[month]-[day] [hour]:[minute]");
		let begin_time = begin_time.format(dt_format).unwrap();
		let end_time = end_time.format(dt_format).unwrap();

		format!("{begin_time} to {end_time}")
	}

	pub fn metric(&self) -> AlertMetric {
		self.result.metric
	}

	pub fn production_value(&self) -> f32 {
		self.result.production_value
	}

	pub fn title(&self) -> String {
		let cadence_and_metric = self.cadence_and_metric();
		let time_range = self.formated_time_range();
		format!("{cadence_and_metric} Alert: {time_range}")
	}

	/// Get the time period this alert covers, returns (begin, end)
	pub fn time_range(&self) -> Result<(OffsetDateTime, OffsetDateTime)> {
		let begin_time = time::OffsetDateTime::from_unix_timestamp(self.timestamp)?;
		let end_time = self.monitor.cadence.add_to_time(begin_time);

		Ok((begin_time, end_time))
	}

	pub fn training_value(&self) -> f32 {
		self.result.training_value
	}
}

/// The various ways to receive alerts

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AlertMethod {
	/// Send an email to the stored address
	#[serde(rename = "email")]
	Email(AlertMethodEmail),
	/// Dump the alert to STDOUT - mostly useful for testing
	#[serde(rename = "stdout")]
	Stdout,
	/// POST the alert to the given URL as a webhook
	#[serde(rename = "webhook")]
	Webhook(AlertMethodWebhook),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AlertMethodEmail {
	pub email: String,
}

impl fmt::Display for AlertMethodEmail {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Email: {}", self.email)
	}
}

impl From<String> for AlertMethodEmail {
	fn from(email: String) -> Self {
		AlertMethodEmail { email }
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AlertMethodWebhook {
	pub url: Url,
}

impl From<Url> for AlertMethodWebhook {
	fn from(url: Url) -> Self {
		AlertMethodWebhook { url }
	}
}

impl TryFrom<String> for AlertMethodWebhook {
	type Error = io::Error;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		match Url::from_str(&value) {
			Ok(url) => Ok(url.into()),
			Err(e) => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				format!("{}", e),
			)),
		}
	}
}

impl fmt::Display for AlertMethodWebhook {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Webhook URL: {}", self.url)
	}
}

impl fmt::Display for AlertMethod {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertMethod::Email(email) => email.to_string(),
			AlertMethod::Stdout => "stdout".to_owned(),
			AlertMethod::Webhook(webhook) => webhook.to_string(),
		};
		write!(f, "{}", s)
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

/// A result from checking a metric
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AlertResult {
	pub metric: AlertMetric,
	pub production_value: f32,
	pub training_value: f32,
	pub difference: f32,
}

impl AlertResult {
	/// Should this result send an alert?
	pub fn exceeds_threshold(&self, tolerance: f32) -> bool {
		self.difference.abs() > tolerance
	}
}

/// Log the completion of an alert handling process
pub async fn write_alert(
	app: &AppState,
	alert_data: Alert,
	monitor_id: Id,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	let mut txn = txn.begin().await?;
	// First, log the alert in the alerts table
	let data = serde_json::to_string(&alert_data)?;
	sqlx::query(
		"
		 insert into alerts
		 	(id, monitor_id, data, date)
		 values
		 	($1, $2, $3, $4)
		",
	)
	.bind(alert_data.id.to_string())
	.bind(monitor_id.to_string())
	.bind(data)
	.bind(alert_data.timestamp)
	.execute(txn.borrow_mut())
	.await?;
	// Then, log a new unsent entry for each AlertMethod in the alert_sends table.
	for method in alert_data.monitor.methods {
		create_alert_send(app, alert_data.id, method, txn.borrow_mut()).await?;
	}
	txn.commit().await?;
	Ok(())
}

impl App {
	/// Retrieve all recorded alerts across all monitors for a given model
	pub async fn get_all_alerts_for_model(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		model_id: Id,
	) -> Result<Vec<Alert>> {
		let rows = sqlx::query(
			"
			select
				alerts.data
			from
				monitors
			join
				alerts
			on
				monitors.id = alerts.monitor_id
			where
				monitors.model_id = $1
		",
		)
		.bind(model_id.to_string())
		.fetch_all(txn.borrow_mut())
		.await?;
		let result = rows
			.iter()
			.map(|row| {
				let data: String = row.get(0);
				let data: Alert = serde_json::from_str(&data).expect("Found malformed alert data");
				data
			})
			.collect();
		Ok(result)
	}

	/// Retrieve a specific alert by id
	pub async fn get_alert(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		alert_id: Id,
	) -> Result<Option<Alert>> {
		Ok(self.state.get_alert(txn, alert_id).await?)
	}
}

impl AppState {
	pub async fn get_alert(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		alert_id: Id,
	) -> Result<Option<Alert>> {
		let result = sqlx::query(
			"
				select
					data
				from
					alerts
				where
					id = $1
			",
		)
		.bind(alert_id.to_string())
		.fetch_optional(txn.borrow_mut())
		.await?;

		if let Some(row) = result {
			let data: String = row.get(0);
			let data: Alert = serde_json::from_str(&data).expect("Malformed alert data");
			Ok(Some(data))
		} else {
			Ok(None)
		}
	}
}
