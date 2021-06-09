use futures::FutureExt;
use std::{sync::Arc, time::Duration};
use tangram_app_common::{
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
	Context,
};
use tangram_error::{err, Result};
use tangram_id::Id;
use tangram_serve::request_id::RequestIdLayer;
use tower::{make::Shared, ServiceBuilder};
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::{error, info, trace_span, Span};
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
	let context_layer = AddExtensionLayer::new(Arc::new(context));
	let request_id_layer = RequestIdLayer::new();
	let trace_layer = TraceLayer::new_for_http()
		.make_span_with(|request: &http::Request<hyper::Body>| {
			let id = request.extensions().get::<Id>().unwrap();
			trace_span!("request", %id)
		})
		.on_request(|request: &http::Request<hyper::Body>, _span: &Span| {
			info!(
				method = %request.method(),
				path = %request.uri().path(),
				query = ?request.uri().query(),
				"request",
			);
		})
		.on_response(
			|response: &http::Response<hyper::Body>, _latency: Duration, _span: &Span| {
				info!(status = %response.status(), "response");
			},
		);
	let service = ServiceBuilder::new()
		.layer(context_layer)
		.layer(request_id_layer)
		.layer(trace_layer)
		.service_fn(handle);
	let addr = std::net::SocketAddr::new(host, port);
	let server = hyper::server::Server::try_bind(&addr)?;
	server.serve(Shared::new(service)).await?;
	Ok(())
}

async fn handle(
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>, http::Error> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let path = request.uri().path();
	let path_components: Vec<_> = path.split('/').skip(1).collect();
	#[rustfmt::skip]
	let response = match path_components.as_slice() {
		["health"] => {
			tangram_app_health::handle(context, request)
		},
		["track"] => {
			tangram_app_track::handle(context, request)
		},
		#[cfg(feature = "tangram_app_login")]
		["login"] => {
			tangram_app_login_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_index")]
		[""] => {
			tangram_app_index_server::handle(context, request)
		},
		#[cfg(feature = "tangram_app_new_repo")]
		["repos", "new"] => {
			tangram_app_new_repo_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_repo_index")]
		["repos", _, ""] => {
			tangram_app_repo_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_repo_edit")]
		["repos", _, "edit"] => {
			tangram_app_repo_edit_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_new_model")]
		["repos", _, "models", "new"] => {
			tangram_app_new_model_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_model_index")]
		["repos", _, "models", _, ""] => {
			tangram_app_model_index_server::handle(context, request)
		}
		["repos", _, "models", _, "download"] => {
			tangram_app_layouts::model_layout::download(context, request)
		}
		#[cfg(feature = "tangram_app_training_grid_index")]
		["repos", _, "models", _, "training_grid", ""] => {
			tangram_app_training_grid_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_grid_item")]
		["repos", _, "models", _, "training_grid", "grid_item", _grid_item_id] => {
			tangram_app_training_grid_item_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_stats_index")]
		["repos", _, "models", _, "training_stats", ""] => {
			tangram_app_training_stats_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_stats_column")]
		["repos", _, "models", _, "training_stats", "columns", _column_name] => {
			tangram_app_training_stats_column_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_playground")]
		["repos", _, "models", _, "playground"] => {
			tangram_app_playground_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_index")]
		["repos", _, "models", _, "training_metrics", ""] => {
			tangram_app_training_metrics_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_class_metrics")]
		["repos", _, "models", _, "training_metrics", "class_metrics"] => {
			tangram_app_training_class_metrics_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_precision_recall")]
		["repos", _, "models", _, "training_metrics", "precision_recall"] => {
			tangram_app_training_metrics_precision_recall_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_training_metrics_roc")]
		["repos", _, "models", _, "training_metrics", "roc"] => {
			tangram_app_training_metrics_roc_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_tuning")]
		["repos", _, "models", _, "tuning"] => {
			tangram_app_tuning_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_predictions_index")]
		["repos", _, "models", _, "production_predictions", ""] => {
			tangram_app_production_predictions_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_prediction")]
		["repos", _, "models", _, "production_predictions", "predictions", _] => {
			tangram_app_production_prediction_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_stats_index")]
		["repos", _, "models", _, "production_stats", ""] => {
			tangram_app_production_stats_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_stats_column")]
		["repos", _, "models", _, "production_stats", "columns", _] => {
			tangram_app_production_stats_column_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_metrics_index")]
		["repos", _, "models", _, "production_metrics", ""] => {
			tangram_app_production_metrics_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_production_class_metrics")]
		["repos", _, "models", _, "production_metrics", "class_metrics"] => {
			tangram_app_production_class_metrics_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_model_edit")]
		["repos", _, "models", _, "edit"] => {
			tangram_app_model_edit_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_user")]
		["user"] => {
			tangram_app_user_server::handle(context, request)
		},
		#[cfg(feature = "tangram_app_new_organization")]
		["organizations", "new"] => {
			tangram_app_new_organization_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_organization_index")]
		["organizations", _, ""] => {
			tangram_app_organization_index_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_edit_organization")]
		["organizations", _, "edit"] => {
			tangram_app_edit_organization_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_new_member")]
		["organizations", _, "members", "new"] => {
			tangram_app_new_member_server::handle(context, request)
		}
		#[cfg(feature = "tangram_app_organization_member")]
		["organizations", _, "members", _] => {
			tangram_app_organization_member_server::handle(context, request)
		}
		_ => async {
			#[cfg(debug_assertions)]
			let dir = std::path::Path::new(env!("OUT_DIR")).join("output");
			#[cfg(not(debug_assertions))]
			let dir = include_out_dir::include_out_dir!("output");
			#[cfg(debug_assertions)]
			let response = tangram_serve::dir::serve_from_dir(&dir, &request).await?;
			#[cfg(not(debug_assertions))]
			let response = tangram_serve::dir::serve_from_include_out_dir(&dir, &request).await?;
			let response = response.unwrap_or_else(|| {
				http::Response::builder()
					.status(http::StatusCode::NOT_FOUND)
					.body(hyper::Body::from("not found"))
					.unwrap()
			});
			Ok(response)
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
	Ok(response)
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
