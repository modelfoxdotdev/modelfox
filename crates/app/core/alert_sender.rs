use crate::{
	heuristics::{
		ALERT_SENDER_HEARTBEAT_DURATION_PRODUCTION, ALERT_SENDER_HEARTBEAT_DURATION_TESTING,
	},
	App, AppState,
};
use anyhow::Result;
use futures::{select, FutureExt};
//use lettre::AsyncTransport;
use std::{borrow::BorrowMut, sync::Arc};
use tangram_id::Id;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum AlertSenderMessage {
	Run(oneshot::Sender<()>),
}

/// Read database for unsent alerts
#[tracing::instrument(level = "info", skip_all)]
pub async fn alert_sender(
	app: Arc<AppState>,
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
		let mut txn = app.begin_transaction().await?;
		let unsent_alerts = app.get_unsent_alerts(txn.borrow_mut()).await?;
		for alert in unsent_alerts {
			app.send_alert(alert, txn.borrow_mut()).await?;
		}
		app.commit_transaction(txn).await?;
		tracing::info!("End alert_sender heartbeat");
		if let Event::Message(AlertSenderMessage::Run(sender)) = event {
			sender.send(()).unwrap();
		}
	}
	Ok(())
}

/// Update an alert record to indicate it has been sent
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
/*
#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		alert::{AlertMethod, AlertMetric},
		monitor::{MonitorCadence, MonitorThreshold, MonitorThresholdMode},
		monitor_checker::MonitorConfig,
		test_common::*,
	};
	use tracing_test::traced_test;
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
			methods: vec![AlertMethod::Email("ben@tangram.dev".to_owned())],
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
	}
}
*/
