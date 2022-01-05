use super::App;
use crate::alerts::{
	check_for_duplicate_monitor, create_monitor, get_monitor, update_monitor, AlertCadence,
	AlertMethod, Monitor, MonitorThreshold,
};
use anyhow::{anyhow, Result};
use tangram_id::Id;

impl App {
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
		model_id: Id,
		config: &MonitorConfig,
	) -> Result<()> {
		let mut db = match self.state.database_pool.begin().await {
			Ok(db) => db,
			Err(_) => return Err(anyhow!("unable to access database pool!")),
		};

		let title = match &config.title {
			Some(s) => s,
			None => "",
		};

		self.create_monitor(CreateMonitorArgs {
			db: &mut db,
			cadence: config.cadence,
			methods: vec![AlertMethod::Stdout].as_slice(),
			model_id,
			threshold: config.threshold,
			title,
		})
		.await?;

		db.commit().await?;
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
