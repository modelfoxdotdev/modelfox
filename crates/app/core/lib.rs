use crate::{
	alert_sender::{alert_sender, AlertSenderMessage},
	clock::Clock,
	monitor_checker::{monitor_checker, MonitorCheckerMessage},
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
};
use anyhow::{anyhow, bail, Result};
use sqlx::postgres::PgPoolOptions;
use std::{path::PathBuf, sync::Arc};
use storage::InMemoryStorage;
use tokio::sync::mpsc;
use url::Url;

pub mod alert;
pub mod alert_sender;
pub mod clock;
pub mod cookies;
pub mod error;
pub mod heuristics;
pub mod model;
pub mod monitor;
pub mod monitor_checker;
pub mod options;
pub mod organizations;
pub mod repos;
pub mod storage;
pub mod timezone;
pub mod track;
pub mod user;

#[cfg(test)]
pub mod test_common;

pub struct App {
	pub state: Arc<AppState>,
	monitor_checker_sender: mpsc::UnboundedSender<MonitorCheckerMessage>,
	alert_sender_sender: mpsc::UnboundedSender<AlertSenderMessage>,
}

#[derive(Debug)]
pub struct AppState {
	pub clock: Clock,
	pub database_pool: sqlx::AnyPool,
	pub options: Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: Storage,
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
			.shared_cache(true)
			.into();
		let pool_max_connections = options.database_max_connections.unwrap_or(10);
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
			StorageOptions::InMemory => Storage::InMemory(InMemoryStorage::new()),
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
			clock: Clock::new(),
			database_pool,
			options,
			smtp_transport,
			storage,
		};
		let state = Arc::new(state);
		let (monitor_checker_sender, monitor_checker_receiver) =
			tokio::sync::mpsc::unbounded_channel();
		let (alert_sender_sender, alert_sender_receiver) = tokio::sync::mpsc::unbounded_channel();
		tokio::spawn({
			let state = Arc::clone(&state);
			async move {
				monitor_checker(state, monitor_checker_receiver)
					.await
					.unwrap();
			}
		});
		tokio::spawn({
			let state = Arc::clone(&state);
			async move {
				alert_sender(state, alert_sender_receiver).await.unwrap();
			}
		});
		let app = App {
			state,
			monitor_checker_sender,
			alert_sender_sender,
		};
		Ok(app)
	}

	pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
		Ok(self.state.begin_transaction().await?)
	}

	pub async fn commit_transaction(&self, txn: sqlx::Transaction<'_, sqlx::Any>) -> Result<()> {
		self.state.commit_transaction(txn).await?;
		Ok(())
	}

	pub fn smtp_transport(&self) -> Option<&lettre::AsyncSmtpTransport<lettre::Tokio1Executor>> {
		self.state.smtp_transport.as_ref()
	}
}

impl AppState {
	pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
		Ok(self.database_pool.begin().await?)
	}

	pub async fn commit_transaction(&self, txn: sqlx::Transaction<'_, sqlx::Any>) -> Result<()> {
		txn.commit().await?;
		Ok(())
	}
}

/// Reset the database state
pub async fn reset_database(database_url: &Option<Url>) -> Result<()> {
	if let Some(database_url) = database_url {
		if database_url.scheme() == "postgres" {
			let pool = PgPoolOptions::new()
				.max_connections(1)
				.connect(database_url.as_str())
				.await?;
			sqlx::query("DROP SCHEMA public CASCADE")
				.execute(&pool)
				.await?;
			sqlx::query("CREATE SCHEMA public").execute(&pool).await?;
			sqlx::query("GRANT ALL ON SCHEMA public TO postgres")
				.execute(&pool)
				.await?;
			sqlx::query("GRANT ALL ON SCHEMA public TO public")
				.execute(&pool)
				.await?;
		}
	}
	Ok(())
}

/// Remove all contents of the data dir, including the database
pub async fn reset_data(database_url: &Option<Url>) -> Result<()> {
	reset_database(database_url).await?;
	std::fs::remove_dir_all(data_path()?)?;
	Ok(())
}

pub fn path_components(request: &http::Request<hyper::Body>) -> Vec<&str> {
	request.uri().path().split('/').skip(1).collect::<Vec<_>>()
}
