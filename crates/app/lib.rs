use anyhow::{anyhow, bail, Result};
use sqlx::prelude::*;
use std::{net::SocketAddr, sync::Arc};
pub use tangram_app_common::options;
use tangram_app_common::{
	alerts::{
		find_current_data, get_all_alerts, get_last_alert_time, AlertCadence, AlertData,
		AlertHeuristics, AlertMethod, AlertMetric, AlertProgress, AlertResult, AlertThresholdMode,
	},
	heuristics::{
		ALERT_METRICS_HEARTBEAT_DURATION_PRODUCTION, ALERT_METRICS_HEARTBEAT_DURATION_TESTING,
		ALERT_METRICS_MINIMUM_TRUE_VALUES_THRESHOLD,
	},
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

/// Manage periodic alerting
async fn alert_manager(context: Arc<Context>) -> Result<()> {
	// TODO - webhook check.
	// Scrape the DB for any incomplete webhook attempts, and spawn an exponential decay thread for any found

	let (begin, period) = if cfg!(not(debug_assertions)) {
		// In release mode, calculate time until next heartbeat
		// Start heartbeat at the top of the hour, run once per hour
		// FIXME - see https://github.com/tangramdotdev/tangram/blob/879c3805e81238e4c30c26725e1bdca5cd0d095e/crates/app/routes/track/server/post.rs#L231
		// that uses chrono, do the same thing with time
		let now = time::OffsetDateTime::now_utc();
		let now_timestamp = now.unix_timestamp();
		let hour = now.hour();
		let next_start = now.replace_time(time::Time::from_hms(hour + 1, 0, 0)?);
		let next_start_timestamp = next_start.unix_timestamp();
		let now_instant = tokio::time::Instant::now();
		let delay = tokio::time::Duration::from_secs(
			(next_start_timestamp - now_timestamp).try_into().unwrap(),
		);
		(
			now_instant + delay,
			ALERT_METRICS_HEARTBEAT_DURATION_PRODUCTION,
		)
	} else {
		// In every mode other than release, don't introduce a delay.  The period is currently 5 seconds.
		(
			tokio::time::Instant::now(),
			ALERT_METRICS_HEARTBEAT_DURATION_TESTING,
		)
	};
	// start interval.
	let mut interval = tokio::time::interval_at(begin, period);

	// Each interval:
	loop {
		interval.tick().await;
		// Grab all currently enabled alerts
		let enabled = get_all_alerts(&context).await?;
		// For each alert:
		for alert in enabled.alerts() {
			let last_run =
				get_last_alert_time(&context, alert.cadence, alert.threshold.metric).await?;
			dbg!(last_run);
			if last_run.is_none() || alert.is_expired(last_run.unwrap()) {
				// Run the alert hueristics if the time has expired for this cadence, or if no heuristics have ever been run
				handle_heuristics(&context, alert).await?;
			}
		}
	}
}

async fn handle_heuristics(context: &Context, heuristics: &AlertHeuristics) -> Result<()> {
	let cadence = heuristics.cadence;
	let metric = heuristics.threshold.metric;
	let already_handled = check_ongoing(cadence, metric, &context.database_pool).await?;

	if already_handled {
		return Ok(());
	}

	let alert_id = write_alert_start(cadence, metric, &context.database_pool).await?;

	let results = check_metrics(heuristics, &context).await?;
	// FIXME - this needs to check against lower and upper
	let exceeded_thresholds: Vec<&AlertResult> = results
		.iter()
		.filter(|r| {
			heuristics
				.get_threshold(r.metric)
				.map(|t| r.exceeds_threshold(t))
				.unwrap_or(false)
		})
		.collect();
	push_alerts(&exceeded_thresholds, &heuristics.methods, context).await?;

	let alert_data = AlertData {
		heuristics: heuristics.to_owned(),
		results: results.to_owned(),
	};
	write_alert_end(alert_id, alert_data, &context.database_pool).await?;

	Ok(())
}

/// Return the current observed values for each heuristic
async fn check_metrics(
	heuristics: &AlertHeuristics,
	context: &Context,
) -> Result<Vec<AlertResult>> {
	let mut results = Vec::new();
	// Look at the current value.
	let current_value =
		find_current_data(heuristics.threshold.metric, heuristics.model_id, context).await?;
	let observed_variance = match heuristics.threshold.mode {
		AlertThresholdMode::Absolute => {
			todo!()
		}
		AlertThresholdMode::Percentage => todo!(),
	};
	let higher = heuristics.threshold.variance_upper;
	// aggregate results

	results.push(AlertResult {
		metric: heuristics.threshold.metric,
		observed_value: current_value,
		observed_variance,
	});
	Ok(results)
}

/// Send alerts containing all exceeded thresholds to each enabled alert method
async fn push_alerts(
	exceeded_thresholds: &[&AlertResult],
	methods: &[AlertMethod],
	context: &Context,
) -> Result<()> {
	for method in methods {
		method.send_alert(exceeded_thresholds, context).await?;
	}
	Ok(())
}

/// Read the DB to see if the given cadence is already in process
async fn check_ongoing(
	cadence: AlertCadence,
	metric: AlertMetric,
	database_pool: &sqlx::AnyPool,
) -> Result<bool> {
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
			alerts.metric = $2 and
			$3 - alerts.date < $4
		",
	)
	.bind(cadence.to_string())
	.bind(metric.to_string())
	.bind(&now)
	.bind(&ten_minutes_in_seconds)
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	Ok(existing.is_some())
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

/// Log the beginning of an alert handling process
async fn write_alert_start(
	cadence: AlertCadence,
	metric: AlertMetric,
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
			   (id, progress, cadence, metric, date)
		  values
			  ($1, $2, $3, $4, $5)
		",
	)
	.bind(id.to_string())
	.bind(AlertProgress::InProgress.to_string())
	.bind(cadence.to_string().to_lowercase())
	.bind(metric.short_name())
	.bind(time::OffsetDateTime::now_utc().unix_timestamp())
	.execute(&mut db)
	.await?;
	db.commit().await?;
	Ok(id)
}
