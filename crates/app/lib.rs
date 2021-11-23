#![feature(async_closure)]
use anyhow::{anyhow, bail, Result};
use std::{fmt, net::SocketAddr, sync::Arc};
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
			alert_manager(context).await.unwrap();
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

impl fmt::Display for AlertCadence {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertCadence::Daily => "daily",
			AlertCadence::Hourly => "hourly",
			AlertCadence::Monthly => "monthly",
			AlertCadence::Weekly => "weekly",
		};
		write!(f, "{}", s)
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
async fn alert_manager(context: Arc<Context>) -> Result<()> {
	// Currently enabled alerts - should probably move up to the context
	let enabled = Alerts(vec![AlertHeuristics {
		cadence: AlertCadence::Hourly,
		thresholds: vec![AlertThreshold {
			metric: AlertMetric::Auc,
			variance: 0.1,
		}],
	}]);
	let enabled = Arc::new(enabled);

	let alert_methods = vec![AlertMethod::Email("ben@tangram.dev".to_owned())];
	let alert_methods = Arc::new(alert_methods);

	let now = tokio::time::Instant::now();
	//let intended_start = ???
	let mut test_interval = tokio::time::interval_at(now, tokio::time::Duration::from_secs(10));
	loop {
		test_interval.tick().await;
		println!("tick");
		handle_alert_cadence(
			&enabled,
			&alert_methods,
			AlertCadence::Hourly,
			&context.database_pool,
		)
		.await?;
	}
}

/// Check the current values for any variences which exceed the thresholds in the heuristics, return any that do
fn check_metrics(heuristics: &AlertHeuristics) -> Vec<AlertResult> {
	println!("Checking metrics!");
	vec![AlertResult {
		metric: AlertMetric::Auc,
		observed_variance: 0.2,
	}]
}

/// Read the DB to see if the given cadence is already in process
async fn check_ongoing(cadence: AlertCadence, database_pool: &sqlx::AnyPool) -> Result<bool> {
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to read database!");
			return Err(anyhow!("Database unavailable"));
		}
	};
	let ten_minutes_in_seconds: i32 = 10 * 60;
	let now = time::OffsetDateTime::now_utc().unix_timestamp();
	let existing = sqlx::query(
		"
			select
				alerts.id as alerts_id
			from alerts
			where
			alerts.cadence = $1 and
			$2 - alerts.date < $3
		",
	)
	.bind(cadence.to_string())
	.bind(&now)
	.bind(&ten_minutes_in_seconds)
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	Ok(existing.is_some())
}

/// Handle a specific alert cadence
async fn handle_alert_cadence(
	alerts: &Alerts,
	alert_methods: &[AlertMethod],
	cadence: AlertCadence,
	database_pool: &sqlx::AnyPool,
) -> Result<()> {
	let already_handled = check_ongoing(cadence, database_pool).await?;

	if already_handled {
		return Ok(());
	}

	let alert_id = write_alert_start(cadence, database_pool).await?;

	let heuristics = alerts.cadence(cadence).unwrap();
	let exceeded_thresholds = check_metrics(heuristics);
	if !exceeded_thresholds.is_empty() {
		push_alerts(&exceeded_thresholds, &alert_methods).await;
	}

	write_alert_end(
		alert_id,
		heuristics,
		&exceeded_thresholds,
		&alert_methods,
		database_pool,
	)
	.await?;
	Ok(())
}

/// Send alerts containing all exceeded thresholds to each enabled alert method
async fn push_alerts(exceeded_thresholds: &[AlertResult], methods: &[AlertMethod]) {
	println!("alerts: {:?}", exceeded_thresholds);
}

/// Log the beginning of an alert handling process
async fn write_alert_start(
	cadence: AlertCadence,
	database_pool: &sqlx::AnyPool,
) -> Result<tangram_id::Id> {
	// Write the current time and cadence being handled
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to write alert progress to DB");
			return Err(anyhow!("Database unavailable"));
		}
	};
	let id = tangram_id::Id::generate();
	sqlx::query(
		"
			insert into alerts
			   (id, progress, cadence, date)
		  values
			  ($1, $2, $3, $4)
		",
	)
	.bind(id.to_string())
	.bind("IN PROGRESS".to_string())
	.bind(cadence.to_string())
	.bind(time::OffsetDateTime::now_utc().unix_timestamp())
	.execute(&mut db)
	.await?;
	db.commit().await?;
	println!("{}", id);
	Ok(id)
}

/// Log the completion of an alert handling process
async fn write_alert_end(
	id: tangram_id::Id,
	heuristics: &AlertHeuristics,
	exceeded_thresholds: &[AlertResult],
	methods: &[AlertMethod],
	database_pool: &sqlx::AnyPool,
) -> Result<()> {
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to write alert progress to DB");
			return Err(anyhow!("Database unavailable"));
		}
	};

	let data = format!("Did some alerting!");

	sqlx::query(
		"
		 update alerts
		 set
		 	progress = $1,
			data = $2
		 where
		 	id = $3
		",
	)
	.bind("COMPLETED".to_string())
	.bind(data)
	.bind(id.to_string())
	.execute(&mut db)
	.await?;
	db.commit().await?;
	Ok(())
}
