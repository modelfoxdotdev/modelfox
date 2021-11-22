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
	tokio::spawn({
		let context = Arc::clone(&context);
		async move {
			alert_manager(context).await;
		}
	});
	tangram_serve::serve(addr, context, handle).await?;
	Ok(())
}

async fn handle(mut request: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
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

/// Alert cadence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertCadence {
	Daily,
	Hourly,
	Monthly,
	Weekly,
}

impl AlertCadence {
	/// Check whether this cadence's deadline is currently being triggered
	fn check_deadline(&self) -> bool {
		let now = time::OffsetDateTime::now_utc();
		// NOTE - do we need microsecond too?  nanosecond?
		let top_of_hour = now.minute() == 0 && now.second() == 0 && now.millisecond() == 0;
		let top_of_day = now.hour() == 0 && top_of_hour;
		let top_of_week = now.weekday() == time::Weekday::Sunday && top_of_day;
		let top_of_month = now.day() == 0 && top_of_day;
		match self {
			&AlertCadence::Daily => top_of_day,
			&AlertCadence::Hourly => top_of_hour,
			&AlertCadence::Monthly => top_of_month,
			AlertCadence::Weekly => top_of_week,
		}
	}
}

/// The various ways to receive alerts
#[derive(Debug, Clone, PartialEq, Eq)]
enum AlertMethod {
	Email(String),
}

/// Statistics that can generate alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertMetric {
	Auc,
	Accuracy,
	MeanSquaredError,
}

/// Single alert threshold
#[derive(Debug, Clone, Copy)]
struct AlertThreshold {
	metric: AlertMetric,
	variance: f32,
}

/// A result from checking a metric
#[derive(Debug, Clone, Copy)]
struct AlertResult {
	metric: AlertMetric,
	observed_variance: f32,
}

/// Thresholds for generating an Alert
#[derive(Debug, Clone)]
struct AlertHeuristics {
	cadence: AlertCadence,
	thresholds: Vec<AlertThreshold>,
}

/// Manager for all enabled alerts
#[derive(Debug, Default)]
struct Alerts(Vec<AlertHeuristics>);

impl Alerts {
	// Retrieve all currently enabled cadences
	fn get_cadences(&self) -> Vec<AlertCadence> {
		self.0
			.iter()
			.map(|ah| ah.cadence)
			.collect::<Vec<AlertCadence>>()
	}

	/// Retrieve the heuristics for the given cadence, if present
	fn cadence(&self, cadence: AlertCadence) -> Option<&AlertHeuristics> {
		for heuristics in &self.0 {
			if heuristics.cadence == cadence {
				return Some(heuristics);
			}
		}
		None
	}
}

/// Manage periodic alerting
async fn alert_manager(context: Arc<Context>) {
	// Currently enabled alerts - should probably move up to the context
	let enabled = Alerts(vec![AlertHeuristics {
		cadence: AlertCadence::Hourly,
		thresholds: vec![AlertThreshold {
			metric: AlertMetric::Auc,
			variance: 0.1,
		}],
	}]);

	let alert_methods = vec![AlertMethod::Email("ben@tangram.dev".to_owned())];

	loop {
		// Are we at a deadline?
		if let Some(cadence) = check_deadline(&enabled.get_cadences()) {
			let already_handled = check_ongoing(cadence).await;

			if already_handled {
				continue;
			}

			write_alert_start(cadence, &context.database_pool).await;

			let heuristics = enabled.cadence(cadence).unwrap();
			let exceeded_thresholds = check_metrics(heuristics);
			if !exceeded_thresholds.is_empty() {
				push_alerts(&exceeded_thresholds, &alert_methods).await;
			}

			write_alert_end(
				heuristics,
				&exceeded_thresholds,
				&alert_methods,
				&context.database_pool,
			)
			.await;
		}
		// sleep 1 second
	}
}

/// Check if it's time to run an alert process
// TODO - should this be a method on Alerts?
fn check_deadline(enabled: &[AlertCadence]) -> Option<AlertCadence> {
	for cadence in enabled {
		if cadence.check_deadline() {
			return Some(*cadence);
		};
	}
	None
}

/// Check the current values for any variences which exceed the thresholds in the heuristics, return any that do
fn check_metrics(heuristics: &AlertHeuristics) -> Vec<AlertResult> {
	todo!()
}

/// Read the DB to see if the given cadence is already in process
async fn check_ongoing(cadence: AlertCadence) -> bool {
	todo!()
}

/// Send alerts containing all exceeded thresholds to each enabled alert method
async fn push_alerts(exceeded_thresholds: &[AlertResult], methods: &[AlertMethod]) {
	todo!()
}

/// Log the beginning of an alert handling process
async fn write_alert_start(cadence: AlertCadence, database_pool: &sqlx::AnyPool) {
	// Write the current time and cadence being handled
	todo!()
}

/// Log the completion of an alert handling process
async fn write_alert_end(
	heuristics: &AlertHeuristics,
	exceeded_thresholds: &[AlertResult],
	methods: &[AlertMethod],
	database_pool: &sqlx::AnyPool,
) {
	todo!()
}
