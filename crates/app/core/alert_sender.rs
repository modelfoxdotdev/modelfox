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
