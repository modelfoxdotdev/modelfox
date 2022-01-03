//! The App presents a Rust API for interacting with Tangram.

use crate::{
	alerts::{
		alert_manager, check_for_duplicate_monitor, create_monitor, get_monitor, update_monitor,
		AlertCadence, AlertMethod, Monitor, MonitorThreshold,
	},
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
};
use anyhow::{anyhow, bail, Result};
use std::{path::PathBuf, sync::Arc};
use tangram_id::Id;
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

impl App {
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
