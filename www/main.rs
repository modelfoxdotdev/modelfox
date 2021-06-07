use clap::Clap;
use pinwheel::prelude::*;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tangram_error::{Error, Result};
use tangram_id::Id;
use tangram_serve::{request_id::RequestIdLayer, RouteMap};
use tangram_www_content::{BlogPost, Content, DocsGuide};
use tower::{make::Shared, ServiceBuilder};
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::{error, info, trace_span, Span};
use tracing_subscriber::prelude::*;

#[derive(Clap)]
enum Args {
	#[clap(name = "serve")]
	Serve,
	#[clap(name = "export")]
	Export(ExportArgs),
}

#[derive(Clap)]
struct ExportArgs {
	path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing()?;
	let args = Args::parse();
	let route_map = route_map();
	match args {
		Args::Serve => {
			let host_from_env = if let Ok(host) = std::env::var("HOST") {
				Some(host.parse()?)
			} else {
				None
			};
			let host = host_from_env.unwrap_or_else(|| "0.0.0.0".parse().unwrap());
			let port_from_env = if let Ok(port) = std::env::var("PORT") {
				Some(port.parse()?)
			} else {
				None
			};
			let port = port_from_env.unwrap_or(8080);
			let context = Context { route_map };
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
		}
		Args::Export(export_args) => {
			let out_dir = std::path::Path::new(env!("OUT_DIR"));
			let cwd = std::env::current_dir()?;
			let dist_path = cwd.join(export_args.path);
			tangram_serve::export::export(route_map, &out_dir, &dist_path)?;
		}
	}
	Ok(())
}

fn route_map() -> RouteMap {
	let mut route_map = RouteMap::new();
	route_map.insert(
		"/".into(),
		Box::new(|| html(tangram_www_index_server::Page::new())),
	);
	route_map.insert(
		"/benchmarks".into(),
		Box::new(|| html(tangram_www_benchmarks_server::Page::new())),
	);
	for slug in BlogPost::slugs().unwrap() {
		route_map.insert(
			format!("/blog/{}", slug).into(),
			Box::new(move || html(tangram_www_blog_post_server::Page::new(slug.clone()))),
		);
	}
	route_map.insert(
		"/blog/".into(),
		Box::new(move || html(tangram_www_blog_index_server::Page::new())),
	);
	route_map.insert(
		"/docs/".into(),
		Box::new(|| html(tangram_www_docs_index_server::Page::new())),
	);
	route_map.insert(
		"/docs/install".into(),
		Box::new(|| html(tangram_www_docs_install_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/".into(),
		Box::new(|| html(tangram_www_docs_getting_started_index_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/train".into(),
		Box::new(|| html(tangram_www_docs_getting_started_train_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_index_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/elixir".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_elixir_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/go".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_go_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/node".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_node_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/python".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_python_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/ruby".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_ruby_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/predict/rust".into(),
		Box::new(|| html(tangram_www_docs_getting_started_predict_rust_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/inspect".into(),
		Box::new(|| html(tangram_www_docs_getting_started_inspect_server::Page::new())),
	);
	route_map.insert(
		"/docs/getting_started/monitor".into(),
		Box::new(|| html(tangram_www_docs_getting_started_monitor_server::Page::new())),
	);
	for slug in DocsGuide::slugs().unwrap() {
		route_map.insert(
			format!("/docs/guides/{}", slug).into(),
			Box::new(move || html(tangram_www_docs_guide_server::Page::new(slug.clone()))),
		);
	}
	route_map.insert(
		"/pricing".into(),
		Box::new(|| html(tangram_www_pricing_server::Page::new())),
	);
	route_map
}

struct Context {
	route_map: RouteMap,
}

async fn handle(
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>, http::Error> {
	let context = request.extensions().get::<Arc<Context>>().unwrap();
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	let response = async {
		if method == http::Method::GET {
			if let Some(page) = context.as_ref().route_map.get(path) {
				let html = page();
				return Result::<_, Error>::Ok(
					http::Response::builder()
						.status(http::StatusCode::OK)
						.body(hyper::Body::from(html))
						.unwrap(),
				);
			}
		}
		if let Some(response) = tangram_serve::serve_from_out_dir!(&request).await? {
			return Ok(response);
		}
		let response = http::Response::builder()
			.status(http::StatusCode::NOT_FOUND)
			.body(hyper::Body::from("not found"))
			.unwrap();
		Ok(response)
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

fn tracing() -> Result<()> {
	let env_layer = tracing_subscriber::EnvFilter::try_from_env("TANGRAM_TRACING");
	let env_layer = if cfg!(debug_assertions) {
		Some(env_layer.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("[]=info")))
	} else {
		env_layer.ok()
	};
	if let Some(env_layer) = env_layer {
		let format_layer = tracing_subscriber::fmt::layer().pretty();
		let subscriber = tracing_subscriber::registry()
			.with(env_layer)
			.with(format_layer);
		subscriber.init();
	}
	Ok(())
}
