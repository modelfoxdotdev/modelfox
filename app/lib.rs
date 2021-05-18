use futures::FutureExt;
use std::sync::Arc;
use tangram_app_common::{
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
	Context,
};
use tangram_error::{err, Result};
use tangram_serve::serve;
use tracing::error;
use tracing_subscriber::prelude::*;
use url::Url;

pub use tangram_app_common::options;

pub fn run(options: Options) -> Result<()> {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_inner(options))
}

async fn run_inner(options: Options) -> Result<()> {
	// Set up tracing.
	tracing()?;
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
	let context = Context {
		database_pool,
		options,
		smtp_transport,
		storage,
	};
	serve(host, port, context, handle).await?;
	Ok(())
}

async fn handle(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> http::Response<hyper::Body> {
	let path = request.uri().path().clone();
	let path_components: Vec<_> = path.split('/').skip(1).collect();
	#[rustfmt::skip]
	let response = match path_components.as_slice() {
		&["health"] => {
			tangram_app_health::handle(context, request)
		},
		&["track"] => {
			tangram_app_track::handle(context, request)
		},
		#[cfg(feature = "tangram_app_login")]
		&["login"] => {
			tangram_app_login_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_index")]
		&[""] => {
			tangram_app_index_server::handle(context, request)
		},
		#[cfg(feature = "tangram_app_new_repo")]
		&["repos", "new"] => {
			tangram_app_new_repo_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_repo_index")]
		&["repos", _, ""] => {
			tangram_app_repo_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_repo_edit")]
		&["repos", _, "edit"] => {
			tangram_app_repo_edit_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_new_model")]
		&["repos", _, "models", "new"] => {
			tangram_app_new_model_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_model_index")]
		 &["repos", _, "models", _, ""] => {
			tangram_app_model_index_server::handle(context, request)
		}
		 &["repos", _, "models", _, "download"] => {
			tangram_app_layouts::model_layout::download(context, request)
		}
		#[cfg(feature = "tangram_app_training_grid_index")]
		 &["repos", _, "models", _, "training_grid", ""] => {
			tangram_app_training_grid_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_grid_item")]
		 &["repos", _, "models", _, "training_grid", "grid_item", _grid_item_id] => {
			tangram_app_training_grid_item_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_stats_index")]
		 &["repos", _, "models", _, "training_stats", ""] => {
			tangram_app_training_stats_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_stats_column")]
		 &["repos", _, "models", _, "training_stats", "columns", _column_name] => {
			tangram_app_training_stats_column_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_playground")]
		 &["repos", _, "models", _, "playground"] => {
			tangram_app_playground_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_index")]
		&["repos", _, "models", _, "training_metrics", ""] => {
			tangram_app_training_metrics_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_class_metrics")]
		 &["repos", _, "models", _, "training_metrics", "class_metrics"] => {
			tangram_app_training_class_metrics_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_precision_recall")]
		 &["repos", _, "models", _, "training_metrics", "precision_recall"] => {
			tangram_app_training_metrics_precision_recall_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_roc")]
		 &["repos", _, "models", _, "training_metrics", "roc"] => {
			tangram_app_training_metrics_roc_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_tuning")]
		 &["repos", _, "models", _, "tuning"] => {
			tangram_app_tuning_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_predictions_index")]
		 &["repos", _, "models", _, "production_predictions", ""] => {
			tangram_app_production_predictions_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_prediction")]
		 &["repos", _, "models", _, "production_predictions", "predictions", _] => {
			tangram_app_production_prediction_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_stats_index")]
		 &["repos", _, "models", _, "production_stats", ""] => {
			tangram_app_production_stats_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_stats_column")]
		 &["repos", _, "models", _, "production_stats", "columns", _] => {
			tangram_app_production_stats_column_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_metrics_index")]
		 &["repos", _, "models", _, "production_metrics", ""] => {
			tangram_app_production_metrics_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_class_metrics")]
		 &["repos", _, "models", _, "production_metrics", "class_metrics"] => {
			tangram_app_production_class_metrics_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_model_edit")]
		&["repos", _, "models", _, "edit"] => {
			tangram_app_model_edit_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_user")]
		&["user"] => {
			tangram_app_user_server::handle(context, request)
		},
		#[cfg(feature = "tangram_app_new_organization")]
		 &["organizations", "new"] => {
			tangram_app_new_organization_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_organization_index")]
		 &["organizations", _, ""] => {
			tangram_app_organization_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_edit_organization")]
		 &["organizations", _, "edit"] => {
			tangram_app_edit_organization_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_new_member")]
		 &["organizations", _, "members", "new"] => {
			tangram_app_new_member_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_organization_member")]
		 &["organizations", _, "members", _] => {
			tangram_app_organization_member_server::handle(context, request)
		}
		_ => async {
			if let Some(response) = tangram_serve::serve_from_out_dir!(&request).await? {
				Ok(response)
			} else {
				let response = http::Response::builder()
					.status(http::StatusCode::NOT_FOUND)
					.body(hyper::Body::from("not found"))
					.unwrap();
				Ok(response)
			}
		}
		.boxed(),
	}
	.await
	.unwrap_or_else(|error| {
		error!(%error);
		http::Response::builder()
			.status(http::StatusCode::INTERNAL_SERVER_ERROR)
			.body(hyper::Body::from("internal server error"))
			.unwrap()
	});
	response
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
		return Err(err!(
			"The database url must start with sqlite: or postgres:."
		));
	};
	let pool = sqlx::any::AnyPoolOptions::default()
		.max_connections(pool_max_connections)
		.connect_with(pool_options)
		.await?;
	Ok(pool)
}

fn tracing() -> Result<()> {
	let env_layer = tracing_subscriber::EnvFilter::try_from_env("TANGRAM_APP_TRACING");
	let env_layer = if cfg!(debug_assertions) {
		Some(env_layer.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("[]=info")))
	} else {
		env_layer.ok()
	};
	if let Some(env_layer) = env_layer {
		if cfg!(debug_assertions) {
			let format_layer = tracing_subscriber::fmt::layer().pretty();
			let subscriber = tracing_subscriber::registry()
				.with(env_layer)
				.with(format_layer);
			subscriber.init();
		} else {
			let journald_layer = tracing_subscriber::fmt::layer().json();
			let subscriber = tracing_subscriber::registry()
				.with(env_layer)
				.with(journald_layer);
			subscriber.init();
		}
	}
	Ok(())
}
