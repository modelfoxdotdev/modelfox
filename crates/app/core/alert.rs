use crate::{
	alert_sender::set_alert_sent,
	monitor::{AlertModelType, Monitor},
	App, AppState,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use std::{borrow::BorrowMut, fmt, io, str::FromStr};
use tangram_id::Id;
use time::{macros::format_description, OffsetDateTime};

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

impl fmt::Display for AlertMethod {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertMethod::Email(addr) => format!("Email: {}", addr),
			AlertMethod::Stdout => "stdout".to_owned(),
			AlertMethod::Webhook(url) => format!("Webhook URL: {}", url),
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
	alert_data: Alert,
	monitor_id: Id,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	let mut txn = txn.begin().await?;
	let data = serde_json::to_string(&alert_data)?;
	sqlx::query(
		"
		 insert into alerts
		 	(id, monitor_id, data, sent, date)
		 values
		 	($1, $2, $3, 0, $4)
		",
	)
	.bind(alert_data.id.to_string())
	.bind(monitor_id.to_string())
	.bind(data)
	.bind(alert_data.timestamp)
	.execute(txn.borrow_mut())
	.await?;
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

impl AppState {
	pub async fn get_unsent_alerts(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	) -> Result<Vec<Alert>> {
		let results = sqlx::query(
			"
				select
					data
				from
					alerts
				where
					sent = 0
			",
		)
		.fetch_all(txn.borrow_mut())
		.await?;
		Ok(results
			.into_iter()
			.map(|row| {
				let alert_data: String = row.get(0);
				let alert_data: Alert =
					serde_json::from_str(&alert_data).expect("Malformed alert data");
				alert_data
			})
			.collect())
	}

	pub async fn send_alert(
		&self,
		alert: Alert,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	) -> Result<()> {
		for method in alert.monitor.methods {
			match method {
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
				AlertMethod::Stdout => println!("exceeded thresholds: {:?}", alert.result),
				AlertMethod::Webhook(_url) => {
					// Spawn a task
					// Attempt the POST, record status in DB.
					// If status has failed, attempt again until it succeeds.

					// multi-step handshake
					// first, confirm DB is idle
					// commit to Sending
					// send
					// commit to Idle
				}
			}
		}
		set_alert_sent(alert.id, txn.borrow_mut()).await?;
		Ok(())
	}
}
