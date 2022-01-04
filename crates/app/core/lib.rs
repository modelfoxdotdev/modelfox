use crate::{
	alerts::alert_manager,
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
};
use anyhow::{anyhow, bail, Result};
use std::{path::PathBuf, sync::Arc};
use tokio::task::JoinHandle;
use url::Url;

pub mod alerts;
pub mod cookies;
pub mod error;
pub mod heuristics;
pub mod model;
pub mod monitor;
pub mod monitor_event;
pub mod options;
pub mod organizations;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod storage;
pub mod timezone;
pub mod track;
pub mod user;

pub struct AppState {
	pub database_pool: sqlx::AnyPool,
	pub options: Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: Storage,
}

pub struct App {
	pub state: Arc<AppState>,
	tasks: Vec<JoinHandle<()>>,
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
	pub async fn new(options: Options) -> Result<Self> {
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
		let state = AppState {
			database_pool,
			options,
			smtp_transport,
			storage,
		};
		let state = Arc::new(state);
		let alert_manager_handle = tokio::spawn({
			let state = Arc::clone(&state);
			async move {
				alert_manager(&state).await.unwrap();
			}
		});
		let app = App {
			state,
			tasks: vec![alert_manager_handle],
		};
		Ok(app)
	}
}

pub fn path_components(request: &http::Request<hyper::Body>) -> Vec<&str> {
	request.uri().path().split('/').skip(1).collect::<Vec<_>>()
}
