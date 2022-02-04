use crate::{
	alert::{write_alert, Alert, AlertMethod, AlertMetric, AlertResult},
	heuristics::{
		ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_DEBUG_THRESHOLD,
		ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_THRESHOLD,
		MONITOR_CHECKER_HEARTBEAT_DURATION_PRODUCTION, MONITOR_CHECKER_HEARTBEAT_DURATION_TESTING,
	},
	model::get_model_bytes,
	monitor::{
		check_for_duplicate_monitor, create_monitor, get_monitor, update_monitor, AlertModelType,
		Monitor, MonitorCadence, MonitorThreshold, MonitorThresholdMode,
	},
	App, AppState,
};
use anyhow::{anyhow, bail, Result};
use futures::FutureExt;
use sqlx::prelude::*;
use std::{borrow::BorrowMut, sync::Arc};
use tangram_app_production_metrics::{ProductionMetrics, ProductionPredictionMetricsOutput};
use tangram_id::Id;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum MonitorCheckerMessage {
	Run(oneshot::Sender<()>),
}

// TODO - monitors should not run immediately, but rather wait one heartbeat.

/// Manage periodic alerting
#[tracing::instrument(level = "info", skip_all)]
pub async fn monitor_checker(
	app: Arc<AppState>,
	mut receiver: mpsc::UnboundedReceiver<MonitorCheckerMessage>,
) -> Result<()> {
	let (begin, period) = if cfg!(not(debug_assertions)) {
		// In release mode, calculate time until next heartbeat
		// Start heartbeat at the top of the hour, run once per hour
		let now = app.clock.now_utc();
		let now_timestamp = now.unix_timestamp();
		let hour = now.hour();
		let next_start = now.replace_time(time::Time::from_hms(hour + 1, 0, 0)?);
		let next_start_timestamp = next_start.unix_timestamp();
		let now_instant = tokio::time::Instant::now();
		let delay = tokio::time::Duration::from_secs(
			(next_start_timestamp - now_timestamp).try_into().unwrap(),
		);
		(
			now_instant + delay + MONITOR_CHECKER_HEARTBEAT_DURATION_PRODUCTION,
			MONITOR_CHECKER_HEARTBEAT_DURATION_PRODUCTION,
		)
	} else {
		// In every mode other than release, don't introduce a delay, just start in one heartbeat.
		(
			tokio::time::Instant::now() + MONITOR_CHECKER_HEARTBEAT_DURATION_TESTING,
			MONITOR_CHECKER_HEARTBEAT_DURATION_TESTING,
		)
	};
	// start interval.
	let mut interval = tokio::time::interval_at(begin, period);
	interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

	// Each interval:
	loop {
		tracing::info!("Begin monitor_checker heartbeat");
		// If we've been notified of shutdown, break the loop.  Otherwise, continue and handle interval
		enum Event {
			Tick,
			Message(MonitorCheckerMessage),
		}
		let event = tokio::select! {
			_ = interval.tick().fuse() => Event::Tick,
			message = receiver.recv().fuse() => match message {
				None => break,
				Some(message) => Event::Message(message),
			},
		};
		let monitors = get_overdue_monitors(&app).await?;
		for monitor in monitors {
			app.check_monitor(&monitor).await?;
		}
		tracing::info!("End monitor_checker heartbeat");
		if let Event::Message(MonitorCheckerMessage::Run(sender)) = event {
			sender.send(()).unwrap();
		}
	}
	Ok(())
}

pub async fn get_overdue_monitors(app_state: &AppState) -> Result<Vec<Monitor>> {
	let mut txn = app_state.begin_transaction().await?;
	let rows = sqlx::query(
		"
			select
				data
			from
				monitors
		",
	)
	.fetch_all(txn.borrow_mut())
	.await?;
	let monitors: Vec<Monitor> = rows
		.iter()
		.map(|row| {
			let monitor: String = row.get(0);
			serde_json::from_str(&monitor).unwrap()
		})
		.collect();
	let mut result = Vec::new();
	// TODO do this in the query, not in Rust.
	for monitor in monitors {
		if monitor
			.is_overdue(txn.borrow_mut(), &app_state.clock)
			.await?
		{
			result.push(monitor);
		}
	}
	app_state.commit_transaction(txn).await?;
	Ok(result)
}

/// Read the model, find the training metric value for the given AlertMetric
pub async fn find_current_training_metric(
	metric: AlertMetric,
	model_id: Id,
	app_state: &AppState,
) -> Result<f32> {
	// Grab the model from the DB
	let bytes = get_model_bytes(&app_state.storage, model_id).await?;
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
						.default_threshold()
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

impl App {
	/// Check a monitor, writing an alert to the DB if necessary
	pub async fn check_monitor(&self, monitor: &Monitor) -> Result<()> {
		self.state.check_monitor(monitor).await?;
		Ok(())
	}

	/// Send a message to the monitor checker and wait for it to reply back indicating it has run.
	#[tracing::instrument(level = "info", skip_all)]
	pub async fn check_monitors(&self) -> Result<()> {
		tracing::info!("starting check_monitors");
		let (sender, receiver) = oneshot::channel();
		self.monitor_checker_sender
			.send(MonitorCheckerMessage::Run(sender))?;
		receiver.await?;
		tracing::info!("finished check_monitors, response received");
		Ok(())
	}

	pub async fn create_monitor(&self, args: CreateMonitorArgs<'_, '_>) -> Result<()> {
		let CreateMonitorArgs {
			db,
			cadence,
			methods,
			model_id,
			threshold,
			title,
		} = args;
		let mut monitor = Monitor {
			cadence,
			id: Id::generate(),
			methods: methods.to_owned(),
			model_id,
			threshold,
			title: title.to_owned(),
		};
		if monitor.title.is_empty() {
			monitor.title = monitor.default_title();
		}
		if check_for_duplicate_monitor(db, &monitor, model_id).await? {
			bail!("Identical alert already exists");
		}
		create_monitor(db, monitor, model_id).await?;
		Ok(())
	}

	pub async fn create_monitor_from_config(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		model_id: Id,
		config: &MonitorConfig,
	) -> Result<()> {
		let mut txn = txn.begin().await?;

		let title = match &config.title {
			Some(s) => s,
			None => "",
		};

		self.create_monitor(CreateMonitorArgs {
			db: &mut txn,
			cadence: config.cadence,
			methods: config.methods.as_slice(),
			model_id,
			threshold: config.threshold,
			title,
		})
		.await?;

		txn.commit().await?;
		Ok(())
	}

	pub async fn update_monitor(&self, args: UpdateMonitorArgs<'_, '_>) -> Result<()> {
		let UpdateMonitorArgs {
			db,
			monitor_id,
			cadence,
			methods,
			model_id,
			threshold,
			title,
		} = args;
		let mut monitor = get_monitor(db, monitor_id).await?;
		let mut title = title.to_owned();
		if title.is_empty() {
			title = monitor.default_title();
		}
		// Replace any components that are different.
		if cadence != monitor.cadence {
			monitor.cadence = cadence;
		}
		if methods != monitor.methods {
			monitor.methods = methods.to_owned();
		}
		if threshold != monitor.threshold {
			monitor.threshold = threshold;
		}
		if title != monitor.title {
			monitor.title = title;
		}
		if check_for_duplicate_monitor(db, &monitor, model_id).await? {
			return Err(anyhow!("Identical alert already exists"));
		}
		update_monitor(db, &monitor, monitor_id, &self.state.clock).await?;
		Ok(())
	}
}

impl AppState {
	pub async fn check_monitor(&self, monitor: &Monitor) -> Result<()> {
		bring_monitor_up_to_date(self, monitor).await?;
		Ok(())
	}
}

pub struct CreateMonitorArgs<'a, 't> {
	pub db: &'a mut sqlx::Transaction<'t, sqlx::Any>,
	pub cadence: MonitorCadence,
	pub methods: &'a [AlertMethod],
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: &'a str,
}

pub struct UpdateMonitorArgs<'a, 't> {
	pub db: &'a mut sqlx::Transaction<'t, sqlx::Any>,
	pub monitor_id: Id,
	pub cadence: MonitorCadence,
	pub methods: &'a [AlertMethod],
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: &'a str,
}

pub struct MonitorConfig {
	pub cadence: MonitorCadence,
	pub threshold: MonitorThreshold,
	pub title: Option<String>,
	pub methods: Vec<AlertMethod>,
}

pub async fn bring_monitor_up_to_date(app_state: &AppState, monitor: &Monitor) -> Result<()> {
	let minimum_metrics_threshold = if cfg!(not(debug_assertions)) {
		ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_THRESHOLD
	} else {
		ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_DEBUG_THRESHOLD
	};

	let mut txn = app_state.begin_transaction().await?;

	let not_enough_existing_metrics =
		get_total_production_metrics(txn.borrow_mut()).await? < minimum_metrics_threshold;
	if not_enough_existing_metrics {
		return Ok(());
	}

	let result = check_metrics(monitor, app_state).await?;
	let exceeded_thresholds: bool = {
		let (upper, lower) = monitor.get_thresholds();
		let upper_exceeded = if let Some(upper) = upper {
			result.difference > upper
		} else {
			false
		};
		let lower_exceeded = if let Some(lower) = lower {
			result.difference.abs() > lower
		} else {
			false
		};
		upper_exceeded || lower_exceeded
	};

	if exceeded_thresholds {
		let alert_data = Alert {
			id: Id::generate(),
			monitor: monitor.to_owned(),
			result: result.to_owned(),
			timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
		};
		write_alert(app_state, alert_data, monitor.id, txn.borrow_mut()).await?;
	}
	app_state.commit_transaction(txn).await?;

	Ok(())
}

/// Return the current observed values for each heuristic
async fn check_metrics(monitor: &Monitor, app_state: &AppState) -> Result<AlertResult> {
	let current_training_value =
		find_current_training_metric(monitor.threshold.metric, monitor.model_id, app_state).await?;
	let mut txn = app_state.begin_transaction().await?;
	let current_production_value =
		get_production_metric(monitor.threshold.metric, monitor.model_id, txn.borrow_mut()).await?;
	if current_production_value.is_none() {
		return Err(anyhow!("Unable to find production metric value"));
	}
	let current_production_value = current_production_value.unwrap();
	let observed_difference = match monitor.threshold.mode {
		MonitorThresholdMode::Absolute => current_production_value - current_training_value,
		MonitorThresholdMode::Percentage => {
			((current_production_value - current_training_value) / current_training_value) * 100.0
		}
	};
	let result = AlertResult {
		metric: monitor.threshold.metric,
		production_value: current_production_value,
		training_value: current_training_value,
		difference: observed_difference,
	};
	// Update monitor last-checked time
	monitor
		.update_timestamp(txn.borrow_mut(), &app_state.clock)
		.await?;
	app_state.commit_transaction(txn).await?;
	Ok(result)
}

/// Retrieve the latest value for the given metric from the production_metrics table
pub async fn get_production_metric(
	metric: AlertMetric,
	model_id: Id,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Option<f32>> {
	let row = sqlx::query(
		"
			select
				data
			from
				production_metrics
			where
				model_id = $1
			order by
				hour
			desc
			limit 1
		",
	)
	.bind(model_id.to_string())
	.fetch_optional(txn.borrow_mut())
	.await?;
	if let Some(row) = row {
		let data: String = row.get(0);
		let production_metrics: ProductionMetrics = serde_json::from_str(&data)?;
		let output = production_metrics.finalize();
		let metrics = output.prediction_metrics;
		if let Some(metrics) = metrics {
			match metrics {
				ProductionPredictionMetricsOutput::Regression(r) => match metric {
					AlertMetric::MeanSquaredError => Ok(Some(r.mse)),
					AlertMetric::RootMeanSquaredError => Ok(Some(r.rmse)),
					_ => Ok(None),
				},
				ProductionPredictionMetricsOutput::BinaryClassification(bc) => match metric {
					AlertMetric::Accuracy => Ok(Some(bc.accuracy)),
					_ => Ok(None),
				},
				ProductionPredictionMetricsOutput::MulticlassClassification(mc) => match metric {
					AlertMetric::Accuracy => Ok(Some(mc.accuracy)),
					_ => Ok(None),
				},
			}
		} else {
			Ok(None)
		}
	} else {
		Ok(None)
	}
}

async fn get_total_production_metrics(txn: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<i64> {
	let result = sqlx::query(
		"
			select
				count(*)
			from production_metrics
		",
	)
	.fetch_one(txn.borrow_mut())
	.await?;
	let result: i64 = result.get(0);
	Ok(result)
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::test_common::*;
	use tracing_test::traced_test;

	#[tokio::test]
	#[traced_test]
	async fn test_monitor_checker() {
		// init app
		let app = init_test_app().await.unwrap();
		app.clock().resume();

		// seed repo and model
		let model_id = init_heart_disease_model(&app).await.unwrap();

		// seed predictions, true values, and monitors
		seed_events(&app, 100, model_id).await.unwrap();
		seed_monitors(&app, model_id).await.unwrap();

		// Ensure the test DB properly registered all four monitors
		let mut txn = app.begin_transaction().await.unwrap();
		let num_monitors = sqlx::query("select count(*) from monitors")
			.fetch_one(txn.borrow_mut())
			.await
			.unwrap();
		let num_monitors: i64 = num_monitors.get(0);
		assert_eq!(num_monitors, 4);
		app.commit_transaction(txn).await.unwrap();

		// Ensure that at the beginning, we have no alerts.
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 0);

		// Scroll half an hour, we should still have no alerts.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 30))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 0);

		// Add another half hour, check that hourly alert is created
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 30))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		dbg!(&all_alerts);
		assert_eq!(all_alerts.len(), 1);

		// Scroll to one day, check that daily alert is created
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60 * 23))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		// 2x hourly
		assert_eq!(all_alerts.len(), 2);

		// Add six more days, check that weekly alert is created
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60 * 24 * 6))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		// 2x hourly, 2x weekly, no daily
		assert_eq!(all_alerts.len(), 4);

		// Add three more weeks, check that monthly alert is created.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60 * 24 * 7 * 3))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		//4x hourly, 2x weekly, no daily, no monthly
		assert_eq!(all_alerts.len(), 6);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_resolve_monitor_lower_absolute() {
		// start an app, seed event, seed monitor, assert alert.
		let app = init_test_app().await.unwrap();
		app.clock().resume();

		let model_id = init_heart_disease_model(&app).await.unwrap();

		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.1),
				difference_upper: None,
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();

		// scroll time, assert first heartbeat generates alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);

		//seed further event that should address monitor
		// The following produces a production_value of 0.8333 repeating
		seed_monitor_event_pair(&app, model_id, false)
			.await
			.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		// scroll to next heartbeat, assert no new alert is created.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_resolve_monitor_upper_absolute() {
		// start an app, seed event, seed monitor, assert alert.
		let app = init_test_app().await.unwrap();
		app.clock().resume();

		let model_id = init_heart_disease_model(&app).await.unwrap();

		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: None,
				difference_upper: Some(0.12),
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();

		// scroll time, assert first heartbeat generates alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);

		//seed further event that should address monitor
		// The following produces a production_value of 0.88 repeating
		seed_monitor_event_pair(&app, model_id, false)
			.await
			.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		// scroll to next heartbeat, assert no new alert is created.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_resolve_monitor_lower_percentage() {
		// start an app, seed event, seed monitor, assert alert.
		let app = init_test_app().await.unwrap();
		app.clock().resume();

		let model_id = init_heart_disease_model(&app).await.unwrap();

		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: Some(15.0),
				difference_upper: None,
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();

		// scroll time, assert first heartbeat generates alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);

		//seed further event that should address monitor
		// The following produces a production_value of 0.8333 repeating
		seed_monitor_event_pair(&app, model_id, false)
			.await
			.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		// scroll to next heartbeat, assert no new alert is created.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_resolve_monitor_upper_percentage() {
		// start an app, seed event, seed monitor, assert alert.
		let app = init_test_app().await.unwrap();
		app.clock().resume();

		let model_id = init_heart_disease_model(&app).await.unwrap();

		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: None,
				difference_upper: Some(15.0),
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();

		// scroll time, assert first heartbeat generates alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);

		//seed further event that should address monitor
		// The following produces a production_value of 0.875
		seed_monitor_event_pair(&app, model_id, false)
			.await
			.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();

		// scroll to next heartbeat, assert no new alert is created.
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		let mut txn = app.begin_transaction().await.unwrap();
		let all_alerts = app
			.get_all_alerts_for_model(txn.borrow_mut(), model_id)
			.await
			.unwrap();
		app.commit_transaction(txn).await.unwrap();
		assert_eq!(all_alerts.len(), 1);
	}
}
