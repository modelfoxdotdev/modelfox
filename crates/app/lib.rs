use anyhow::{bail, Result};
use std::{net::SocketAddr, sync::Arc};
pub use tangram_app_common::options;
use tangram_app_common::{
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
	Context,
};
use tracing::error;
use url::Url;

pub fn run(options: Options) -> Result<()> {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_inner(options))
}

async fn run_inner(options: Options) -> Result<()> {
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
	// Start the server.
	let host = options.host;
	let port = options.port;
	let addr = SocketAddr::new(host, port);
	let context = Context {
		database_pool,
		options,
		smtp_transport,
		storage,
		sunfish: sunfish::init!(),
	};
	let context = Arc::new(context);
	tangram_serve::serve(addr, context, handle).await?;
	Ok(())
}

async fn handle(mut request: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let context = context.clone();
	let response = context
		.sunfish
		.handle(&mut request)
		.await
		.unwrap_or_else(|error| {
			error!(%error, backtrace = %error.backtrace());
			Some(
				http::Response::builder()
					.status(http::StatusCode::INTERNAL_SERVER_ERROR)
					.body(hyper::Body::from("internal server error"))
					.unwrap(),
			)
		});
	response.unwrap_or_else(|| {
		http::Response::builder()
			.status(http::StatusCode::NOT_FOUND)
			.body(hyper::Body::from("not found"))
			.unwrap()
	})
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
