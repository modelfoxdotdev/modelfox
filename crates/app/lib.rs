use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use std::{fmt, net::SocketAddr, sync::Arc};
pub use tangram_app_common::options;
use tangram_app_common::{
	heuristics::ALERT_METRICS_MINIMUM_TRUE_VALUES_THRESHOLD,
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
	Context,
};
use tangram_app_production_metrics::{ProductionMetrics, ProductionPredictionMetrics};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
enum AlertCadence {
	#[serde(rename = "daily")]
	Daily,
	#[serde(rename = "hourly")]
	Hourly,
	#[serde(rename = "monthly")]
	Monthly,
	#[serde(rename = "weekly")]
	Weekly,
}

impl AlertCadence {
	pub fn duration(&self) -> tokio::time::Duration {
		match self {
			AlertCadence::Daily => tokio::time::Duration::from_secs(60 * 60 * 24),
			AlertCadence::Hourly => tokio::time::Duration::from_secs(60 * 60),
			AlertCadence::Monthly => tokio::time::Duration::from_secs(60 * 60 * 24 * 31), //FIXME that's not correct
			AlertCadence::Weekly => tokio::time::Duration::from_secs(60 * 60 * 24 * 7),
		}
	}
}

impl Default for AlertCadence {
	fn default() -> Self {
		AlertCadence::Hourly
	}
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
// FIXME - using tag = type and renaming here causes sqlx to panic!!
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
enum AlertMethod {
	Email(String),
}

/// Statistics that can generate alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
enum AlertMetric {
	#[serde(rename = "accuracy")]
	Accuracy,
	#[serde(rename = "root_mean_squared_error")]
	RootMeanSquaredError,
}

/// An alert record can be in one of these states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlertProgress {
	Completed,
	InProgress,
}

impl fmt::Display for AlertProgress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			AlertProgress::Completed => "COMPLETED",
			AlertProgress::InProgress => "IN PROGRESS",
		};
		write!(f, "{}", s)
	}
}

/// Single alert threshold
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct AlertThreshold {
	metric: AlertMetric,
	variance: f32,
}

/// A result from checking a metric
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct AlertResult {
	metric: AlertMetric,
	observed_value: f32,
	observed_variance: f32,
}

impl AlertResult {
	/// Should this result send an alert?
	pub fn exceeds_threshold(&self, tolerance: f32) -> bool {
		self.observed_variance.abs() > tolerance
	}
}

/// Thresholds for generating an Alert
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct AlertHeuristics {
	cadence: AlertCadence,
	thresholds: Vec<AlertThreshold>,
}

impl AlertHeuristics {
	/// Retrieve the variance tolerance for a given metric, if present
	fn get_threshold(&self, metric: AlertMetric) -> Option<f32> {
		for threshold in &self.thresholds {
			if threshold.metric == metric {
				return Some(threshold.variance);
			}
		}
		None
	}
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

/// Collection for the alert results from a single run
#[derive(Debug, Default, Deserialize, Serialize)]
struct AlertData {
	alert_methods: Vec<AlertMethod>,
	heuristics: AlertHeuristics,
	results: Vec<AlertResult>,
}

/// Manage periodic alerting
async fn alert_manager(context: Arc<Context>) -> Result<()> {
	// Currently enabled alerts - should probably move up to the context
	let enabled = Alerts(vec![
		AlertHeuristics {
			cadence: AlertCadence::Hourly,
			thresholds: vec![AlertThreshold {
				metric: AlertMetric::Accuracy,
				variance: 0.2,
			}],
		},
		AlertHeuristics {
			cadence: AlertCadence::Daily,
			thresholds: vec![AlertThreshold {
				metric: AlertMetric::Accuracy,
				variance: 0.1,
			}],
		},
	]);
	let enabled = Arc::new(enabled);

	let alert_methods = vec![AlertMethod::Email("ben@tangram.dev".to_owned())];
	let alert_methods = Arc::new(alert_methods);

	let now = tokio::time::Instant::now();
	//let intended_start = ???
	let enabled_cadences = enabled.get_cadences();
	// Spawn a subtask for each cadence
	for cadence in enabled_cadences {
		tokio::spawn({
			let alert_methods = Arc::clone(&alert_methods);
			let context = Arc::clone(&context);
			let enabled = Arc::clone(&enabled);
			let mut interval = tokio::time::interval_at(now, cadence.duration());
			async move {
				loop {
					interval.tick().await;
					handle_alert_cadence(&enabled, &alert_methods, cadence, &context.database_pool)
						.await
						.unwrap();
				}
			}
		});
	}
	Ok(())
}

/// Return the current observed values for each heuristic
async fn check_metrics(
	heuristics: &AlertHeuristics,
	database_pool: &sqlx::AnyPool,
) -> Result<Vec<AlertResult>> {
	println!("Checking metrics!");

	// cases:
	// 1. This is at least the second alert.  Check against the previous answer.
	// 2. This is the first alert.  There is no possible alert to submit.

	let mut results = Vec::new();
	let previous_alert_data = find_previous_alert_data(heuristics.cadence, database_pool).await?;
	if previous_alert_data.is_none() {
		return Ok(vec![]);
	}
	for threshold in &heuristics.thresholds {
		// Look at the current value.
		let current_value = find_current_data(threshold.metric, database_pool).await?;
		if current_value.is_none() {
			// Not enough predictions have been logged - abort!
			return Ok(vec![]);
		}
		let current_value = current_value.unwrap();
		results.push(AlertResult {
			metric: threshold.metric,
			observed_value: current_value,
			observed_variance: (current_value - threshold.variance).abs(),
		});
	}
	Ok(results)
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
				alerts.id
			from alerts
			where
			alerts.cadence = $1 and
			$2 - alerts.date < $3
		",
	)
	.bind(cadence.to_string())
	.bind(&now)
	.bind(&ten_minutes_in_seconds)
	//.bind(&5) // TODO remove - just for testing!
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	Ok(existing.is_some())
}

/// Find the most recent value for the given metric
async fn find_current_data(
	metric: AlertMetric,
	database_pool: &sqlx::AnyPool,
) -> Result<Option<f32>> {
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to read database!");
			return Err(anyhow!("Database unavailable"));
		}
	};

	// First, check how many true values have been logged.  If under the configured threshold, don't do anything.
	let num_true_values_result = sqlx::query("select count(*) from true_values")
		.fetch_one(&mut db)
		.await?;
	let num_true_values: i64 = num_true_values_result.get(0);
	if num_true_values < ALERT_METRICS_MINIMUM_TRUE_VALUES_THRESHOLD {
		return Ok(None);
	}

	let result = sqlx::query(
		"
			select
				data
			from
				production_metrics
		",
	)
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;

	match result {
		Some(r) => {
			let data: String = r.get(0);
			let res: ProductionMetrics = serde_json::from_str(&data)?;
			match metric {
				AlertMetric::Accuracy => {
					let accuracy = match res.prediction_metrics {
						ProductionPredictionMetrics::BinaryClassification(bcppm) => {
							bcppm.finalize().unwrap().accuracy
						}
						ProductionPredictionMetrics::MulticlassClassification(mccppm) => {
							mccppm.finalize().unwrap().accuracy
						}
						_ => unreachable!(), // we will never have regression for the Accuracy metric
					};
					Ok(Some(accuracy))
				}
				AlertMetric::RootMeanSquaredError => {
					let rmse = match res.prediction_metrics {
						ProductionPredictionMetrics::Regression(rppm) => {
							rppm.finalize().unwrap().rmse
						}
						_ => unreachable!(), // RMSE is only for regression
					};
					Ok(Some(rmse))
				}
			}
		}
		None => Ok(None),
	}
}

/// Find the previous instance of the given alert cadence
async fn find_previous_alert_data(
	cadence: AlertCadence,
	database_pool: &sqlx::AnyPool,
) -> Result<Option<AlertData>> {
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to read database!");
			return Err(anyhow!("Database unavailable"));
		}
	};
	let row = sqlx::query(
		"
			select
				data
			from
				alerts
			where
				cadence = $1 and
				date = (select max(date) from alerts where progress = $2)
		",
	)
	.bind(cadence.to_string())
	.bind(AlertProgress::Completed.to_string())
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	match row {
		Some(r) => {
			let data: String = r.get("data");
			let result: AlertData = serde_json::from_str(&data)?;
			Ok(Some(result))
		}
		None => Ok(None),
	}
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
	let results = check_metrics(heuristics, database_pool).await?;
	println!("here");
	let exceeded_thresholds: Vec<&AlertResult> = results
		.iter()
		.filter(|r| {
			heuristics
				.get_threshold(r.metric)
				.map(|t| r.exceeds_threshold(t))
				.unwrap_or(false)
		})
		.collect();
	push_alerts(&exceeded_thresholds, alert_methods).await;

	let alert_data = AlertData {
		alert_methods: alert_methods.to_owned(),
		heuristics: heuristics.to_owned(),
		results: results.to_owned(),
	};
	write_alert_end(alert_id, alert_data, database_pool).await?;
	Ok(())
}

/// Send alerts containing all exceeded thresholds to each enabled alert method
async fn push_alerts(exceeded_thresholds: &[&AlertResult], methods: &[AlertMethod]) {
	for method in methods {
		println!("pushing alert to {:?}: {:?}", method, exceeded_thresholds);
	}
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
	.bind(AlertProgress::InProgress.to_string())
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
	alert_data: AlertData,
	database_pool: &sqlx::AnyPool,
) -> Result<()> {
	let mut db = match database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to write alert progress to DB");
			return Err(anyhow!("Database unavailable"));
		}
	};

	let data = serde_json::to_string(&alert_data)?;

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
	.bind(AlertProgress::Completed.to_string())
	.bind(data)
	.bind(id.to_string())
	.execute(&mut db)
	.await?;
	db.commit().await?;
	Ok(())
}
