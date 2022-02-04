use crate::{
	alert::{Alert, AlertMethod},
	heuristics::{
		ALERT_SENDER_HEARTBEAT_DURATION_PRODUCTION, ALERT_SENDER_HEARTBEAT_DURATION_TESTING,
		ALERT_SENDER_MAXIMUM_RETRY_PERIODS, ALERT_SENDER_RETRY_DECAY_FACTOR,
		ALERT_SENDER_RETRY_INITIAL_PERIOD,
	},
	App, AppState,
};
use anyhow::Result;
use futures::{select, FutureExt};
use sqlx::prelude::*;
use std::{borrow::BorrowMut, io, str::FromStr, sync::Arc};
use tangram_id::Id;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum AlertSenderMessage {
	Run(oneshot::Sender<()>),
}

/// Read database for unsent alerts
#[tracing::instrument(level = "info", skip_all)]
pub async fn alert_sender(
	app_state: Arc<AppState>,
	mut receiver: mpsc::UnboundedReceiver<AlertSenderMessage>,
) -> Result<()> {
	let period = if cfg!(debug_assertions) {
		ALERT_SENDER_HEARTBEAT_DURATION_TESTING
	} else {
		ALERT_SENDER_HEARTBEAT_DURATION_PRODUCTION
	};
	let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
	interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
	loop {
		enum Event {
			Tick,
			Message(AlertSenderMessage),
		}
		let event = select! {
			_ = interval.tick().fuse() => Event::Tick,
			message = receiver.recv().fuse() => match message {
				None => break,
				Some(message) => Event::Message(message),
			}
		};
		tracing::info!("Begin alert_sender heartbeat");
		let mut txn = app_state.begin_transaction().await?;
		// First, find any orphaned tasks still marked Sending and reset them
		reset_dropped_sends(&app_state, txn.borrow_mut()).await?;
		let unsent_alerts = get_all_unsent_alert_sends(&app_state, txn.borrow_mut()).await?;
		for alert_send in unsent_alerts {
			handle_alert_send_with_decay(&app_state, alert_send, txn.borrow_mut()).await?;
		}
		app_state.commit_transaction(txn).await?;
		tracing::info!("End alert_sender heartbeat");
		if let Event::Message(AlertSenderMessage::Run(sender)) = event {
			sender.send(()).unwrap();
		}
	}
	Ok(())
}

pub async fn handle_alert_send(
	app_state: &AppState,
	alert_send: &AlertSend,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<AlertSendStatus> {
	set_alert_send_status(
		app_state,
		alert_send.id,
		AlertSendStatus::Sending,
		txn.borrow_mut(),
	)
	.await?;
	increment_attempt_count(alert_send.id, txn.borrow_mut()).await?;
	let exceeded_thresholds = alert_send.alert.result;

	match &alert_send.method {
		AlertMethod::Email(email) => {
			let email = lettre::Message::builder()
				.from("Tangram <noreply@tangram.dev>".parse()?)
				.to(email.email.parse()?)
				.subject("Tangram Metrics Alert")
				.body(format!(
					"Exceeded alert thresholds: {:?}",
					exceeded_thresholds
				))?;
			let status = match app_state.send_email(email).await {
				Ok(_) => AlertSendStatus::Succeeded,
				Err(_) => AlertSendStatus::Retrying,
			};
			set_alert_send_status(app_state, alert_send.id, status, txn.borrow_mut()).await?;
			Ok(status)
		}
		AlertMethod::Stdout => {
			println!("exceeded thresholds: {:?}", exceeded_thresholds);
			Ok(AlertSendStatus::Succeeded)
		}
		AlertMethod::Webhook(url) => {
			let url = &url.url;
			let status = match app_state
				.http_sender
				.post_payload(exceeded_thresholds, url.clone())
				.await
			{
				Ok(_) => AlertSendStatus::Succeeded,
				Err(_) => AlertSendStatus::Retrying,
			};
			set_alert_send_status(app_state, alert_send.id, status, txn.borrow_mut()).await?;
			Ok(status)
		}
	}
}

async fn reset_dropped_sends(
	app_state: &AppState,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	// Find any record in alert_sends that as a status of Sending and an initiated time greater than one heartbeat ago
	let one_heartbeat_ago =
		(app_state.clock().now_utc() - ALERT_SENDER_HEARTBEAT_DURATION_PRODUCTION).unix_timestamp();
	let rows = sqlx::query(
		"
			select
				id
			from
				alert_sends
			where
				status = $1
			and
				initiated_date <= $2
		",
	)
	.bind(u8::from(AlertSendStatus::Sending) as i64)
	.bind(one_heartbeat_ago)
	.fetch_all(txn.borrow_mut())
	.await?;
	// Set all statuses back to Unsent, so they're picked up in this heartbeat.
	for row in rows {
		let id: String = row.get(0);
		let id = Id::from_str(&id)?;
		set_alert_send_status(app_state, id, AlertSendStatus::Unsent, txn.borrow_mut()).await?;
	}
	Ok(())
}

// TODO - call this at the start for all alerts.
async fn handle_alert_send_with_decay(
	app_state: &AppState,
	alert_send: AlertSend,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	// this should either spawn a task, or be spawned in a task?
	let mut retry_count = 0u32;
	while retry_count < ALERT_SENDER_MAXIMUM_RETRY_PERIODS {
		// attempt the send
		if (handle_alert_send(app_state, &alert_send, txn.borrow_mut()).await).is_ok() {
			set_alert_send_status(
				app_state,
				alert_send.id,
				AlertSendStatus::Succeeded,
				txn.borrow_mut(),
			)
			.await?;
			return Ok(());
		}
		// If we failed, set back to retrying
		retry_count += 1;
		set_alert_send_status(
			app_state,
			alert_send.id,
			AlertSendStatus::Retrying,
			txn.borrow_mut(),
		)
		.await?;
		// change period
		let old_secs = ALERT_SENDER_RETRY_INITIAL_PERIOD.as_secs();
		let new_secs = old_secs * ALERT_SENDER_RETRY_DECAY_FACTOR.pow(retry_count);
		let period = std::time::Duration::from_secs(new_secs);
		tokio::time::sleep(period).await;
	}
	// If we hit the end and it never succeeded, write a failiure
	set_alert_send_status(
		app_state,
		alert_send.id,
		AlertSendStatus::Failed,
		txn.borrow_mut(),
	)
	.await?;
	Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSendStatus {
	Unsent,
	Sending,
	Retrying,
	Succeeded,
	Failed,
}

impl TryFrom<u8> for AlertSendStatus {
	type Error = io::Error;
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(AlertSendStatus::Unsent),
			1 => Ok(AlertSendStatus::Sending),
			2 => Ok(AlertSendStatus::Retrying),
			3 => Ok(AlertSendStatus::Succeeded),
			4 => Ok(AlertSendStatus::Failed),
			_ => Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"Unrecognized alert attempt status",
			)),
		}
	}
}

impl From<AlertSendStatus> for u8 {
	fn from(status: AlertSendStatus) -> Self {
		match status {
			AlertSendStatus::Unsent => 0,
			AlertSendStatus::Sending => 1,
			AlertSendStatus::Retrying => 2,
			AlertSendStatus::Succeeded => 3,
			AlertSendStatus::Failed => 4,
		}
	}
}

/// Insert a new alert attempt
pub async fn create_alert_send(
	app_state: &AppState,
	alert_id: Id,
	method: AlertMethod,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Id> {
	let id = Id::generate();
	let now = app_state.clock().now_utc().unix_timestamp();
	let method_json = serde_json::to_string(&method)?;
	sqlx::query(
		"
		insert into alert_sends
			(id, alert_id, attempt_count, method, status, initiated_date)
		values
			($1, $2, 0, $3, $4, $5)
	",
	)
	.bind(id.to_string())
	.bind(alert_id.to_string())
	.bind(method_json)
	.bind(u8::from(AlertSendStatus::Unsent) as i64)
	.bind(now)
	.execute(txn.borrow_mut())
	.await?;
	Ok(id)
}

async fn increment_attempt_count(
	alert_id: Id,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	sqlx::query(
		"
			update
				alert_sends
			set
				attempt_count = attempt_count + 1
			where
				id = $1
		",
	)
	.bind(alert_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

/// Update the status of an alert attempt
pub async fn set_alert_send_status(
	app_state: &AppState,
	alert_send_id: Id,
	new_status: AlertSendStatus,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	sqlx::query(
		"
			update
				alert_sends
			set
				status = $1
			where
				id = $2
		",
	)
	.bind(u8::from(new_status) as i64)
	.bind(alert_send_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	// If the attempt status is either Succeeded or Failed, also set the completed time
	if matches!(new_status, AlertSendStatus::Succeeded)
		|| matches!(new_status, AlertSendStatus::Failed)
	{
		let now = app_state.clock().now_utc().unix_timestamp();
		sqlx::query(
			"
				update
					alert_sends
				set
					completed_date = $1
				where
					id = $2
			",
		)
		.bind(now)
		.bind(alert_send_id.to_string())
		.execute(txn.borrow_mut())
		.await?;
	}
	Ok(())
}

pub struct AlertSend {
	id: Id,
	alert: Alert,
	method: AlertMethod,
}

pub async fn get_all_unsent_alert_sends(
	app_state: &AppState,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<AlertSend>> {
	let rows = sqlx::query(
		"
		select
			id,
			alert_id,
			method
		from alert_sends
		where
			status = $1
	",
	)
	.bind(u8::from(AlertSendStatus::Unsent) as i64)
	.fetch_all(txn.borrow_mut())
	.await?;
	// Process all returned rows with just the alert id
	struct RawAlertSend {
		id: Id,
		alert_id: Id,
		method: AlertMethod,
	}
	let raw_results: Vec<RawAlertSend> = rows
		.into_iter()
		.map(|row| {
			// Pull data
			let id: String = row.get(0);
			let id = Id::from_str(&id).unwrap();
			let alert_id: String = row.get(1);
			let alert_id = Id::from_str(&alert_id).unwrap();
			let method: String = row.get(2);
			let method: AlertMethod =
				serde_json::from_str(&method).expect("Malformed alert method");
			RawAlertSend {
				id,
				alert_id,
				method,
			}
		})
		.collect();
	// Grab the actual alert for each returned Id
	let mut results = vec![];
	for result in raw_results {
		let alert = app_state
			.get_alert(txn.borrow_mut(), result.alert_id)
			.await?
			.unwrap();
		results.push(AlertSend {
			id: result.id,
			alert,
			method: result.method,
		});
	}
	Ok(results)
}

/// Update an alert record to indicate it has been picked up by the alert_sender
pub async fn set_alert_sent(
	alert_id: Id,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<()> {
	sqlx::query(
		"
			update
				alerts
			set
				sent = 1
			where
				id = $1
		",
	)
	.bind(alert_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

impl App {
	/// Send a message to the alert sender and wait for it to reply back indicating it has run.
	pub async fn send_alerts(&self) -> Result<()> {
		let (sender, receiver) = oneshot::channel();
		self.alert_sender_sender
			.send(AlertSenderMessage::Run(sender))?;
		receiver.await?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		alert::{AlertMethod, AlertMethodWebhook, AlertMetric},
		monitor::{MonitorCadence, MonitorThreshold, MonitorThresholdMode},
		monitor_checker::MonitorConfig,
		test_common::*,
	};
	use tracing_test::traced_test;

	async fn get_total_sends_with_status(
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		status: AlertSendStatus,
	) -> Result<i64> {
		let result = sqlx::query(
			"
				select
					count (*)
				from
					alert_sends
				where
					status = $1
			",
		)
		.bind(u8::from(status) as i64)
		.fetch_one(txn.borrow_mut())
		.await?
		.get(0);
		Ok(result)
	}

	#[tokio::test]
	#[traced_test]
	async fn test_alert_email_send() {
		// Seed app, generate an alert, assert the email is sent
		let app = init_test_app().await.unwrap();
		app.clock().resume();
		let model_id = init_heart_disease_model(&app).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.05),
				difference_upper: Some(0.05),
			},
			title: None,
			methods: vec![AlertMethod::Email("ben@tangram.dev".to_owned().into())],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();
		seed_events(&app, 100, model_id).await.unwrap();
		// Scroll to allow monitor_checker to write alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();

		// Scroll to allow alert_sender to pick up alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(10))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();

		// Assert exactly one success has been logged.
		let mut txn = app.begin_transaction().await.unwrap();
		let num_successes =
			get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Succeeded)
				.await
				.unwrap();
		assert_eq!(num_successes, 1);

		// Assert there are no unset, sending, or failed
		let num_unsent = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Unsent)
			.await
			.unwrap();
		assert_eq!(num_unsent, 0);
		let num_sending = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Sending)
			.await
			.unwrap();
		assert_eq!(num_sending, 0);
		let num_failed = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Failed)
			.await
			.unwrap();
		assert_eq!(num_failed, 0);
		app.commit_transaction(txn).await.unwrap();
	}

	#[tokio::test]
	#[traced_test]
	async fn test_alert_webhook_send() {
		// Seed app, generate an alert, assert the email is sent
		let app = init_test_app().await.unwrap();
		app.clock().resume();
		let model_id = init_heart_disease_model(&app).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.05),
				difference_upper: Some(0.05),
			},
			title: None,
			methods: vec![AlertMethod::Webhook(
				AlertMethodWebhook::try_from("http://0.0.0.0:8085/webhook".to_owned()).unwrap(),
			)],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();
		seed_events(&app, 100, model_id).await.unwrap();
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		// Scroll to allow alert_sender to pick up alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(10))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();

		// Assert exactly one success has been logged.
		let mut txn = app.begin_transaction().await.unwrap();
		let num_successes =
			get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Succeeded)
				.await
				.unwrap();
		assert_eq!(num_successes, 1);

		// Assert there are no unset, sending, or failed
		let num_unsent = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Unsent)
			.await
			.unwrap();
		assert_eq!(num_unsent, 0);
		let num_sending = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Sending)
			.await
			.unwrap();
		assert_eq!(num_sending, 0);
		let num_failed = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Failed)
			.await
			.unwrap();
		assert_eq!(num_failed, 0);
		app.commit_transaction(txn).await.unwrap();
	}

	#[tokio::test]
	#[traced_test]
	async fn test_alert_multiple_methods_send() {
		// Seed app, generate an alert, assert the email is sent
		let app = init_test_app().await.unwrap();
		app.clock().resume();
		let model_id = init_heart_disease_model(&app).await.unwrap();
		seed_monitor_event_pair(&app, model_id, true).await.unwrap();
		let test_monitor = MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.05),
				difference_upper: Some(0.05),
			},
			title: None,
			methods: vec![
				AlertMethod::Email("ben@tangram.dev".to_owned().into()),
				AlertMethod::Webhook(
					AlertMethodWebhook::try_from("http://0.0.0.0:8085/webhook".to_owned()).unwrap(),
				),
			],
		};
		seed_single_monitor(&app, &test_monitor, model_id)
			.await
			.unwrap();
		seed_events(&app, 100, model_id).await.unwrap();
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(60 * 60))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();
		// Scroll to allow alert_sender to pick up alert
		app.clock().pause();
		app.clock()
			.advance(std::time::Duration::from_secs(10))
			.await;
		app.clock().resume();
		app.check_monitors().await.unwrap();

		// Assert exactly two successes have been logged.
		let mut txn = app.begin_transaction().await.unwrap();
		let num_successes =
			get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Succeeded)
				.await
				.unwrap();
		assert_eq!(num_successes, 2);

		// Assert there are no unset, sending, or failed
		let num_unsent = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Unsent)
			.await
			.unwrap();
		assert_eq!(num_unsent, 0);
		let num_sending = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Sending)
			.await
			.unwrap();
		assert_eq!(num_sending, 0);
		let num_failed = get_total_sends_with_status(txn.borrow_mut(), AlertSendStatus::Failed)
			.await
			.unwrap();
		assert_eq!(num_failed, 0);
		app.commit_transaction(txn).await.unwrap();
	}
}
