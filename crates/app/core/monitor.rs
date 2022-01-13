use super::{App, AppState};
use crate::alerts::{
	check_for_duplicate_monitor, create_monitor, get_monitor, update_monitor, AlertCadence,
	AlertMethod, Monitor, MonitorThreshold, bring_monitor_up_to_date
};
use anyhow::{anyhow, Result};
use sqlx::Acquire;
use tangram_id::Id;

impl App {
	/// Check a monitor, writing an alert to the DB if necessary
	pub async fn check_monitor(&self, monitor: &Monitor) -> Result<()> {
		self.state.check_monitor(monitor).await?;
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
			return Err(anyhow!("Identical alert already exists"));
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
			methods: vec![AlertMethod::Stdout].as_slice(),
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
		update_monitor(db, &monitor, monitor_id).await?;
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
	pub cadence: AlertCadence,
	pub methods: &'a [AlertMethod],
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: &'a str,
}

pub struct UpdateMonitorArgs<'a, 't> {
	pub db: &'a mut sqlx::Transaction<'t, sqlx::Any>,
	pub monitor_id: Id,
	pub cadence: AlertCadence,
	pub methods: &'a [AlertMethod],
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: &'a str,
}

pub struct MonitorConfig {
	pub cadence: AlertCadence,
	pub threshold: MonitorThreshold,
	pub title: Option<String>,
}
