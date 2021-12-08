use anyhow::{anyhow, bail, Result};
use sqlx::prelude::*;
use std::{net::SocketAddr, sync::Arc};
pub use tangram_app_common::options;
use tangram_app_common::{
	alerts::{
		AlertCadence, AlertData, AlertHeuristics, AlertMethod, AlertMetric, AlertProgress,
		AlertResult, AlertThreshold, Alerts,
	},
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

/// Manage periodic alerting
async fn alert_manager(context: Arc<Context>) -> Result<()> {
	// TODO - this will all be grabbed from the DB, configured by the user
	// Currently enabled alerts - should probably move up to the context
	let enabled = Alerts::from(vec![AlertHeuristics {
		cadence: AlertCadence::Testing,
		threshold: AlertThreshold {
			metric: AlertMetric::Accuracy,
			variance: 0.1,
		},
	}]);
	/*
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
	*/
	let enabled = Arc::new(enabled);

	let alert_methods = vec![
		//AlertMethod::Email("ben@tangram.dev".to_owned()),
		AlertMethod::Stdout,
	];
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
					handle_alert_cadence(&enabled, &alert_methods, cadence, &context)
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
	let mut results = Vec::new();
	let previous_alert_data = find_previous_alert_data(heuristics.cadence, database_pool).await?;
	if previous_alert_data.is_none() {
		return Ok(vec![]);
	}
	// Look at the current value.
	let current_value = find_current_data(heuristics.threshold.metric, database_pool).await?;
	if current_value.is_none() {
		// Not enough predictions have been logged - abort!
		return Ok(vec![]);
	}
	let current_value = current_value.unwrap();
	results.push(AlertResult {
		metric: heuristics.threshold.metric,
		observed_value: current_value,
		observed_variance: (current_value - heuristics.threshold.variance).abs(),
	});
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
	//.bind(&ten_minutes_in_seconds)
	.bind(&5) // TODO remove - just for testing!
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
	context: &Context,
) -> Result<()> {
	let already_handled = check_ongoing(cadence, &context.database_pool).await?;

	if already_handled {
		return Ok(());
	}

	let alert_id = write_alert_start(cadence, &context.database_pool).await?;

	let heuristics = alerts.cadence(cadence).unwrap();
	let results = check_metrics(heuristics, &context.database_pool).await?;
	let exceeded_thresholds: Vec<&AlertResult> = results
		.iter()
		.filter(|r| {
			heuristics
				.get_threshold(r.metric)
				.map(|t| r.exceeds_threshold(t))
				.unwrap_or(false)
		})
		.collect();
	push_alerts(&exceeded_thresholds, alert_methods, context).await?;

	let alert_data = AlertData {
		alert_methods: alert_methods.to_owned(),
		heuristics: heuristics.to_owned(),
		results: results.to_owned(),
	};
	write_alert_end(alert_id, alert_data, &context.database_pool).await?;
	Ok(())
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
