//! The App presents a Rust API for interacting with Tangram.

use crate::{
	alerts::{
		alert_manager, check_for_duplicate_monitor, create_monitor, get_monitor, update_monitor,
		AlertCadence, AlertMethod, Monitor, MonitorThreshold,
	},
	model::get_model_bytes,
	monitor_event::{
		BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
		NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
		TrueValueMonitorEvent,
	},
	options::{Options, StorageOptions},
	production_metrics::ProductionMetrics,
	production_stats::ProductionStats,
	storage::{LocalStorage, S3Storage, Storage},
};
use anyhow::{anyhow, bail, Result};
use chrono::prelude::*;
use memmap::Mmap;
use num::ToPrimitive;
use sqlx::prelude::*;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tangram_id::Id;
use tracing::error;
use url::Url;

pub struct App {
	pub database_pool: sqlx::AnyPool,
	pub options: Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: Storage,
}

pub struct CreateMonitorArgs<'a, 't> {
	pub db: &'a mut sqlx::Transaction<'t, sqlx::Any>,
	pub cadence: AlertCadence,
	pub methods: &'a [AlertMethod],
	pub model_id: Id,
	pub threshold: MonitorThreshold,
	pub title: &'a str,
}

struct CreateDatabasePoolOptions {
	pub database_max_connections: Option<u32>,
	pub database_url: Url,
}

/// Create the database pool.
async fn create_database_pool(options: CreateDatabasePoolOptions) -> Result<sqlx::AnyPool> {
	let database_url = options.database_url.to_string();
	let (pool_options, pool_max_connections) = if database_url.starts_with("sqlite:") {
		let pool_options = database_url
			.parse::<sqlx::sqlite::SqliteConnectOptions>()?
			.create_if_missing(true)
			.foreign_keys(true)
			.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
			.into();
		let pool_max_connections = options.database_max_connections.unwrap_or(1);
		(pool_options, pool_max_connections)
	} else if database_url.starts_with("postgres:") {
		let pool_options = database_url
			.parse::<sqlx::postgres::PgConnectOptions>()?
			.into();
		let pool_max_connections = options.database_max_connections.unwrap_or(10);
		(pool_options, pool_max_connections)
	} else {
		bail!("The database url must start with sqlite: or postgres:.");
	};
	let pool = sqlx::any::AnyPoolOptions::default()
		.max_connections(pool_max_connections)
		.connect_with(pool_options)
		.await?;
	Ok(pool)
}

pub fn migrate(database_url: Url) -> Result<()> {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(migrate_inner(database_url))
}

pub async fn migrate_inner(database_url: Url) -> Result<()> {
	let database_pool = create_database_pool(CreateDatabasePoolOptions {
		database_max_connections: Some(1),
		database_url,
	})
	.await?;
	tangram_app_migrations::run(&database_pool).await?;
	Ok(())
}

/// Retrieve the user cache directory using the `dirs` crate.
pub fn cache_path() -> Result<PathBuf> {
	let cache_dir =
		dirs::cache_dir().ok_or_else(|| anyhow!("failed to find user cache directory"))?;
	let tangram_cache_dir = cache_dir.join("tangram");
	std::fs::create_dir_all(&tangram_cache_dir).map_err(|_| {
		anyhow!(
			"failed to create tangram cache directory in {}",
			tangram_cache_dir.display()
		)
	})?;
	Ok(tangram_cache_dir)
}

/// Retrieve the user data directory using the `dirs` crate.
pub fn data_path() -> Result<PathBuf> {
	let data_dir = dirs::data_dir().ok_or_else(|| anyhow!("failed to find user data directory"))?;
	let tangram_data_dir = data_dir.join("tangram");
	std::fs::create_dir_all(&tangram_data_dir).map_err(|_| {
		anyhow!(
			"failed to create tangram data directory in {}",
			tangram_data_dir.display()
		)
	})?;
	Ok(tangram_data_dir)
}

/// Retrieve the default database url, which is a sqlite database in the user data directory.
pub fn default_database_url() -> Url {
	let tangram_database_path = data_path().unwrap().join("db").join("tangram.db");
	std::fs::create_dir_all(tangram_database_path.parent().unwrap()).unwrap();
	let url = format!(
		"sqlite:{}",
		tangram_database_path.to_str().unwrap().to_owned()
	);
	Url::parse(&url).unwrap()
}
impl App {
	pub async fn new(options: Options) -> Result<Arc<Self>> {
		// Create the database pool.
		let database_pool = create_database_pool(CreateDatabasePoolOptions {
			database_max_connections: options.database.max_connections,
			database_url: options.database.url.clone(),
		})
		.await?;
		if tangram_app_migrations::empty(&database_pool).await? {
			// Run all migrations if the database is empty.
			tangram_app_migrations::run(&database_pool).await?;
		} else {
			// If the database is not empty, verify that all migrations have already been run.
			tangram_app_migrations::verify(&database_pool).await?;
		}
		// Create the smtp transport.
		let smtp_transport = if let Some(smtp) = options.smtp.as_ref() {
			Some(
				lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp.host)?
					.credentials((&smtp.username, &smtp.password).into())
					.build(),
			)
		} else {
			None
		};
		let storage = match options.storage.clone() {
			StorageOptions::Local(options) => Storage::Local(LocalStorage { path: options.path }),
			StorageOptions::S3(options) => Storage::S3(S3Storage::new(
				options.access_key,
				options.secret_key,
				options.endpoint,
				options.bucket,
				options.region,
				options.cache_path,
			)?),
		};
		let app = App {
			database_pool,
			options,
			smtp_transport,
			storage,
		};
		let app = Arc::new(app);
		tokio::spawn({
			let app = Arc::clone(&app);
			async move {
				alert_manager(&app).await.unwrap();
			}
		});
		Ok(app)
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

	pub async fn track_events(&self, events: Vec<MonitorEvent>) -> Result<()> {
		let mut db = match self.database_pool.begin().await {
			Ok(db) => db,
			Err(_) => return Err(anyhow!("unable to access database pool")),
		};
		let mut model_cache = BTreeMap::new();
		for event in events {
			match event {
				MonitorEvent::Prediction(monitor_event) => {
					let handle_prediction_result = handle_prediction_monitor_event(
						&mut db,
						&self.storage,
						&mut model_cache,
						monitor_event,
					)
					.await;
					if let Err(e) = handle_prediction_result {
						error!(%e);
						return Err(anyhow!("{}", e));
					}
				}
				MonitorEvent::TrueValue(monitor_event) => {
					let handle_true_value_result = handle_true_value_monitor_event(
						&mut db,
						&self.storage,
						&mut model_cache,
						monitor_event,
					)
					.await;
					if handle_true_value_result.is_err() {
						return Err(anyhow!(
							"{}",
							handle_true_value_result.err().unwrap().to_string()
						));
					}
				}
			}
		}
		db.commit().await?;
		Ok(())
	}
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

pub async fn handle_prediction_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	model_cache: &mut BTreeMap<Id, Mmap>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let bytes = match model_cache.get(&model_id) {
		Some(bytes) => bytes,
		None => {
			let model = get_model_bytes(data_storage, model_id).await?;
			model_cache.insert(model_id, model);
			model_cache.get(&model_id).unwrap()
		}
	};
	let model = tangram_model::from_bytes(bytes)?;
	write_prediction_monitor_event(db, model_id, &monitor_event).await?;
	insert_or_update_production_stats_for_monitor_event(db, model_id, model, monitor_event).await?;
	Ok(())
}

pub async fn write_prediction_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &PredictionMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string();
	let row = sqlx::query(
		"
			select count(*) from predictions
			where
				model_id = $1
				and identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier.to_string())
	.fetch_one(&mut *db)
	.await?;
	let prediction_count: i64 = row.get(0);
	if prediction_count > 0 {
		bail!("A prediction has already been logged with this identifier.");
	}
	let prediction_monitor_event_id = Id::generate();
	let date = &monitor_event.date;
	let input = serde_json::to_string(&monitor_event.input)?;
	let output = serde_json::to_string(&monitor_event.output)?;
	let options = serde_json::to_string(&monitor_event.options)?;
	sqlx::query(
		"
			insert into predictions
				(id, model_id, date, identifier, input, options, output)
			values
				($1, $2, $3, $4, $5, $6, $7)
		",
	)
	.bind(&prediction_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
	.bind(&input)
	.bind(&options)
	.bind(&output)
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn handle_true_value_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	model_cache: &mut BTreeMap<Id, Mmap>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let bytes = match model_cache.get(&model_id) {
		Some(model) => model,
		None => {
			let model = get_model_bytes(data_storage, monitor_event.model_id).await?;
			model_cache.insert(model_id, model);
			model_cache.get(&model_id).unwrap()
		}
	};
	let model = tangram_model::from_bytes(bytes)?;
	write_true_value_monitor_event(db, model_id, &monitor_event).await?;
	insert_or_update_production_metrics_for_monitor_event(db, model_id, model, monitor_event)
		.await?;
	Ok(())
}

pub async fn write_true_value_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string();
	let row = sqlx::query(
		"
			select count(*) from true_values
			where
				model_id = $1
				and identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier.to_string())
	.fetch_one(&mut *db)
	.await?;
	let true_value_count: i64 = row.get(0);
	if true_value_count > 0 {
		bail!("A prediction has already been logged with this identifier.");
	}
	let true_value_monitor_event_id = Id::generate();
	let date = monitor_event.date;
	let true_value = &monitor_event.true_value.to_string();
	sqlx::query(
		"
			insert into true_values
				(id, model_id, date, identifier, value)
			values
				($1, $2, $3, $4, $5)
		",
	)
	.bind(&true_value_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
	.bind(&true_value.to_string())
	.execute(&mut *db)
	.await?;
	Ok(())
}

pub async fn insert_or_update_production_stats_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: tangram_model::ModelReader<'_>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let date = monitor_event.date;
	let hour = Utc
		.ymd(date.year(), date.month(), date.day())
		.and_hms(date.hour(), 0, 0);
	let rows = sqlx::query(
		"
			select
				data
			from production_stats
			where
				model_id = $1
				and hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(&mut *db)
	.await?;
	if let Some(row) = rows.get(0) {
		let data: String = row.get(0);
		let mut production_stats: ProductionStats = serde_json::from_str(&data)?;
		production_stats.update(model, monitor_event);
		let data = serde_json::to_string(&production_stats)?;
		sqlx::query(
			"
				update
					production_stats
				set
					data = $1
				where
					model_id = $2
					and hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_stats = ProductionStats::new(model, start_date, end_date);
		production_stats.update(model, monitor_event);
		let data = serde_json::to_string(&production_stats)?;
		sqlx::query(
			"
				insert into production_stats
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&data)
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}

pub async fn insert_or_update_production_metrics_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: tangram_model::ModelReader<'_>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string().to_string();
	let rows = sqlx::query(
		"
			select
				output,
				date
			from
				predictions
			where
				predictions.model_id = $1
				and predictions.identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_all(&mut *db)
	.await?;
	if rows.is_empty() {
		bail!("Failed to find prediction with identifier {}", identifier);
	}
	let true_value = match &monitor_event.true_value {
		serde_json::Value::Number(value) => {
			NumberOrString::Number(value.as_f64().unwrap().to_f32().unwrap())
		}
		serde_json::Value::String(value) => NumberOrString::String(value.clone()),
		_ => unimplemented!(),
	};
	let row = rows
		.get(0)
		.ok_or_else(|| anyhow!("Failed to find prediction with identifier {}", identifier))?;
	let output: String = row.get(0);
	let date: i64 = row.get(1);
	let date = Utc.timestamp(date, 0);
	let hour = date
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.with_nanosecond(0)
		.unwrap();
	let output: PredictOutput = serde_json::from_str(&output)?;
	let prediction = match output {
		PredictOutput::Regression(RegressionPredictOutput { value }) => {
			NumberOrString::Number(value)
		}
		PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
		PredictOutput::MulticlassClassification(MulticlassClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
	};
	let row = sqlx::query(
		"
			select
				data
			from production_metrics
			where
				model_id = $1
				and hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_optional(&mut *db)
	.await?;
	if let Some(row) = row {
		let data: String = row.get(0);
		let mut production_metrics: ProductionMetrics = serde_json::from_str(&data)?;
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_string(&production_metrics)?;
		sqlx::query(
			"
				update
					production_metrics
				set
					data = $1
				where
					model_id = $2
					and hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_metrics = ProductionMetrics::new(model, start_date, end_date);
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_string(&production_metrics)?;
		sqlx::query(
			"
				insert into production_metrics
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&data)
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}
