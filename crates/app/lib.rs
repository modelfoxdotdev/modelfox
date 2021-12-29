use anyhow::{anyhow, bail, Result};
use sqlx::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tangram_app_context::Context;
pub use tangram_app_core::options;
use tangram_app_core::{
	alerts::{
		find_current_data, get_overdue_monitors, AlertData, AlertMethod, AlertMetric, AlertResult,
		Monitor, MonitorThresholdMode,
	},
	heuristics::{
		ALERT_METRICS_HEARTBEAT_DURATION_PRODUCTION, ALERT_METRICS_HEARTBEAT_DURATION_TESTING,
		ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_THRESHOLD,
	},
	options::{Options, StorageOptions},
	production_metrics::{ProductionMetrics, ProductionPredictionMetricsOutput},
	storage::{LocalStorage, S3Storage, Storage},
	App,
};
use tangram_id::Id;
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
	let app = App {
		database_pool,
		options,
		smtp_transport,
		storage,
	};
	let context = Context::new(app, sunfish::init!());
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
		let enabled = get_overdue_monitors(&context.app).await?;
		// TODO get_overdue_alerts: "which alerts are currently ready to be processed" - if last run is more than one cadence period ago
		// For each alert:
		for monitor in enabled.alerts() {
			handle_monitor(&context.app, monitor).await?;
		}
	}
}

async fn handle_monitor(app: &App, monitor: &Monitor) -> Result<()> {
	// TODO Do separate for dev and release mode
	let not_enough_existing_metrics = get_total_production_metrics(app).await?
		< ALERT_METRICS_MINIMUM_PRODUCTION_METRICS_THRESHOLD;
	if not_enough_existing_metrics {
		return Ok(());
	}

	let already_handled = check_ongoing(monitor.id, &app.database_pool).await?;
	if already_handled {
		return Ok(());
	}

	let results = check_metrics(monitor, app).await?;
	let exceeded_thresholds: Vec<&AlertResult> = results
		.iter()
		.filter(|r| {
			let (upper, lower) = monitor.get_thresholds();
			let upper_exceeded = if let Some(u) = upper {
				r.observed_variance > u
			} else {
				false
			};
			let lower_exceeded = if let Some(l) = lower {
				r.observed_variance < l
			} else {
				false
			};
			upper_exceeded || lower_exceeded
		})
		.collect();
	push_alerts(&exceeded_thresholds, &monitor.methods, app).await?;

	let alert_data = AlertData {
		preference: monitor.to_owned(),
		results: results.to_owned(),
	};
	write_alert(alert_data, monitor.id, &app.database_pool).await?;

	Ok(())
}

/// Return the current observed values for each heuristic
async fn check_metrics(preference: &Monitor, context: &App) -> Result<Vec<AlertResult>> {
	let mut results = Vec::new();
	let current_training_value =
		find_current_data(preference.threshold.metric, preference.model_id, context).await?;
	let current_production_value =
		get_production_metric(preference.threshold.metric, preference.model_id, context).await?;
	if current_production_value.is_none() {
		return Err(anyhow!("Unable to find production metric value"));
	}
	let current_production_value = current_production_value.unwrap();
	let observed_variance = match preference.threshold.mode {
		MonitorThresholdMode::Absolute => current_training_value - current_production_value,
		MonitorThresholdMode::Percentage => todo!(),
	};

	results.push(AlertResult {
		metric: preference.threshold.metric,
		observed_value: current_training_value,
		observed_variance,
	});
	Ok(results)
}

/// Retrieve the latest value for the given metric from the production_metrics table
pub async fn get_production_metric(
	metric: AlertMetric,
	model_id: Id,
	context: &App,
) -> Result<Option<f32>> {
	let mut db = match context.database_pool.begin().await {
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
				production_metrics
			where
				model_id = $1
			order by
				hour
			desc
			limit 1
		",
	)
	.bind(model_id.to_string())
	.fetch_optional(&mut db)
	.await?;
	if let Some(row) = row {
		let data: String = row.get(0);
		let production_metrics: ProductionMetrics = serde_json::from_str(&data)?;
		let output = production_metrics.finalize();
		let metrics = output.prediction_metrics;
		if let Some(metrics) = metrics {
			use ProductionPredictionMetricsOutput::*;
			match metrics {
				Regression(r) => match metric {
					AlertMetric::MeanSquaredError => Ok(Some(r.mse)),
					AlertMetric::RootMeanSquaredError => Ok(Some(r.rmse)),
					_ => Ok(None),
				},
				BinaryClassification(bc) => match metric {
					AlertMetric::Accuracy => Ok(Some(bc.accuracy)),
					_ => Ok(None),
				},
				MulticlassClassification(mc) => match metric {
					AlertMetric::Accuracy => Ok(Some(mc.accuracy)),
					_ => Ok(None),
				},
			}
		} else {
			Ok(None)
		}
	} else {
		Ok(None)
	}
}

async fn get_total_production_metrics(context: &App) -> Result<i64> {
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => {
			eprintln!("Oh no! Failed to read database!");
			return Err(anyhow!("Database unavailable"));
		}
	};
	let result = sqlx::query(
		"
			select
				count(*)
			from production_metrics
		",
	)
	.fetch_one(&mut db)
	.await?;
	db.commit().await?;
	let result: i64 = result.get(0);
	Ok(result)
}

/// Send alerts containing all exceeded thresholds to each enabled alert method
async fn push_alerts(
	exceeded_thresholds: &[&AlertResult],
	methods: &[AlertMethod],
	context: &App,
) -> Result<()> {
	for method in methods {
		method.send_alert(exceeded_thresholds, context).await?;
	}
	Ok(())
}

/// Read the DB to see if the given cadence is already in process
async fn check_ongoing(id: Id, database_pool: &sqlx::AnyPool) -> Result<bool> {
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
				*
			from
				alerts
			where
				id = $1 and
				$2 - alerts.date < $3
		",
	)
	.bind(id.to_string())
	.bind(&now)
	.bind(&ten_minutes_in_seconds)
	.fetch_optional(&mut db)
	.await?;
	db.commit().await?;
	Ok(existing.is_some())
}

/// Log the completion of an alert handling process
async fn write_alert(
	alert_data: AlertData,
	monitor_id: Id,
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
		 insert into alerts
		 	(id, monitor_id, data, date)
		 values
		 	($1, $2, $3, $4)
		",
	)
	.bind(Id::generate().to_string())
	.bind(monitor_id.to_string())
	.bind(data)
	.execute(&mut db)
	.await?;
	db.commit().await?;
	Ok(())
}
