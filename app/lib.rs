use futures::FutureExt;
use std::{collections::BTreeMap, sync::Arc};
use tangram_app_common::{
	options::{Options, StorageOptions},
	storage::{LocalStorage, S3Storage, Storage},
	Context,
};
use tangram_error::{err, Result};
use tangram_serve::serve;
use url::Url;

mod migrations;

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
	if self::migrations::empty(&database_pool).await? {
		// Run all migrations if the database is empty.
		self::migrations::run(&database_pool).await?;
	} else {
		// If the database is not empty, verify that all migrations have already been run.
		self::migrations::verify(&database_pool).await?;
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
		StorageOptions::S3(options) => Storage::S3(S3Storage {
			access_key: options.access_key,
			secret_key: options.secret_key,
			endpoint: options.endpoint,
			bucket: options.bucket,
			region: options.region,
			cache_path: options.cache_path,
		}),
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
	serve(host, port, context, request_handler).await?;
	Ok(())
}

async fn request_handler(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> http::Response<hyper::Body> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	let query = path_and_query.query();
	let path_components: Vec<_> = path.split('/').skip(1).collect();
	let search_params: Option<BTreeMap<String, String>> = query.map(|search_params| {
		url::form_urlencoded::parse(search_params.as_bytes())
			.into_owned()
			.collect()
	});
	#[rustfmt::skip]
	let response = match (&method, path_components.as_slice()) {
		(&http::Method::GET, &["health"]) => {
			tangram_api::health::get(&context, request).boxed()
		},
		(&http::Method::POST, &["track"]) => {
			tangram_api::track::post(&context, request).boxed()
		},
		(&http::Method::GET, &["login"]) => {
			tangram_app_login_server::get(&context, request, search_params).boxed()
		}
		(&http::Method::POST, &["login"]) => {
			tangram_app_login_server::post(&context, request).boxed()
		}
		(&http::Method::GET, &[""]) => {
			tangram_app_index_server::get(&context, request).boxed()
		},
		(&http::Method::GET, &["repos", "new"]) => {
			tangram_app_new_repo_server::get(&context, request).boxed()
		}
		(&http::Method::POST, &["repos", "new"]) => {
			tangram_app_new_repo_server::post(&context, request).boxed()
		}
		(&http::Method::GET, &["repos", repo_id, ""]) => {
			tangram_app_repo_index_server::get(&context, request, repo_id).boxed()
		}
		(&http::Method::GET, &["repos", repo_id, "edit"]) => {
			tangram_app_repo_edit_server::get(&context, request, repo_id).boxed()
		}
		(&http::Method::POST, &["repos", repo_id, "edit"]) => {
			tangram_app_repo_edit_server::post(&context, request, repo_id).boxed()
		}
		(&http::Method::GET, &["repos", repo_id, "models", "new"]) => {
			tangram_app_new_model_server::get(&context, request, repo_id).boxed()
		}
		(&http::Method::POST, &["repos", repo_id, "models", "new"]) => {
			tangram_app_new_model_server::post(&context, request, repo_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, ""]) => {
			tangram_app_model_index_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "download"]) => {
			tangram_app_layouts::model_layout::download(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_grid", ""]) => {
			tangram_app_training_grid_index_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_grid", "grid_item", grid_item_id]) => {
			tangram_app_training_grid_item_server::get(&context, request, model_id, grid_item_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", ""]) => {
			tangram_app_training_stats_index_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_stats", "columns", column_name]) => {
			tangram_app_training_stats_column_server::get(&context, request, model_id, column_name).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "playground"]) => {
			tangram_app_playground_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", ""]) => {
			tangram_app_training_metrics_index_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "class_metrics"]) => {
			tangram_app_training_class_metrics_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "precision_recall"]) => {
			tangram_app_training_metrics_precision_recall_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "training_metrics", "roc"]) => {
			tangram_app_training_metrics_roc_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "tuning"]) => {
			tangram_app_tuning_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			tangram_app_production_predictions_index_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::POST, &["repos", _repo_id, "models", model_id, "production_predictions", ""]) => {
			tangram_app_production_predictions_index_server::post(&context, request, model_id).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_predictions", "predictions", identifier]) => {
			tangram_app_production_prediction_server::get(&context, request, model_id, identifier).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", ""]) => {
			tangram_app_production_stats_index_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_stats", "columns", column_name],
		) => {
			tangram_app_production_stats_column_server::get(&context, request, model_id, column_name, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", ""]) => {
			tangram_app_production_metrics_index_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "production_metrics", "class_metrics"]) => {
			tangram_app_production_class_metrics_server::get(&context, request, model_id, search_params).boxed()
		}
		(&http::Method::GET, &["repos", _repo_id, "models", model_id, "edit"]) => {
			tangram_app_model_edit_server::get(&context, request, model_id).boxed()
		}
		(&http::Method::POST, &["repos", repo_id, "models", model_id, "edit"]) => {
			tangram_app_model_edit_server::post(&context, request, repo_id, model_id).boxed()
		}
		(&http::Method::GET, &["user"]) => {
			tangram_app_user_server::get(&context, request).boxed()
		},
		(&http::Method::POST, &["user"]) => {
			tangram_app_user_server::post(&context, request).boxed()
		}
		(&http::Method::GET, &["organizations", "new"]) => {
			tangram_app_new_organization_server::get(&context, request).boxed()
		}
		(&http::Method::POST, &["organizations", "new"]) => {
			tangram_app_new_organization_server::post(&context, request).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, ""]) => {
			tangram_app_organization_index_server::get(&context, request, organization_id).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, ""]) => {
			tangram_app_organization_index_server::post(&context, request, organization_id).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, "edit"]) => {
			tangram_app_edit_organization_server::get(&context, request, organization_id).boxed()
		}
		(&http::Method::GET, &["organizations", organization_id, "members", "new"]) => {
			tangram_app_new_member_server::get(&context, request, organization_id).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, "members", "new"]) => {
			tangram_app_new_member_server::post(&context, request, organization_id).boxed()
		}
		(&http::Method::POST, &["organizations", organization_id, "edit"]) => {
			tangram_app_edit_organization_server::post(&context, request, organization_id).boxed()
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
		eprintln!("{}", error);
		let body = if cfg!(debug_assertions) {
			format!("{}", error)
		} else {
			"internal server error".to_owned()
		};
		http::Response::builder()
			.status(http::StatusCode::INTERNAL_SERVER_ERROR)
			.body(hyper::Body::from(body))
			.unwrap()
	});
	eprintln!("{} {} {}", method, path, response.status());
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
	migrations::run(&database_pool).await?;
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
			"DATABASE_URL must be a sqlite or postgres database url"
		));
	};
	let pool = sqlx::any::AnyPoolOptions::default()
		.max_connections(pool_max_connections)
		.connect_with(pool_options)
		.await?;
	Ok(pool)
}
