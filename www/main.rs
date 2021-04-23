use clap::Clap;
use html::html;
use std::{path::PathBuf, sync::Arc};
use tangram_error::{Error, Result};
use tangram_serve::{serve, RouteMap};

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
	let args = Args::parse();
	let route_map = route_map();
	match args {
		Args::Serve => {
			let host = "0.0.0.0".parse()?;
			let port = 8080;
			let context = Context { route_map };
			serve(host, port, context, request_handler).await?;
		}
		Args::Export(export_args) => {
			let out_dir = std::path::Path::new(env!("OUT_DIR"));
			let cwd = std::env::current_dir()?;
			let dist_path = cwd.join(export_args.path);
			tangram_serve::export(&out_dir, &dist_path, route_map)?;
		}
	}
	Ok(())
}

fn route_map() -> RouteMap {
	let mut route_map = RouteMap::new();
	route_map.insert("/", &|| {
		html!(<tangram_www_index_server::Page />).render_to_string()
	});
	route_map.insert("/docs/", &|| {
		html!(<tangram_www_docs_index_server::Page />).render_to_string()
	});
	route_map.insert("/docs/install", &|| {
		html!(<tangram_www_docs_install_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/", &|| {
		html!(<tangram_www_docs_getting_started_index_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/train", &|| {
		html!(<tangram_www_docs_getting_started_train_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/", &|| {
		html!(<tangram_www_docs_getting_started_predict_index_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/elixir", &|| {
		html!(<tangram_www_docs_getting_started_predict_elixir_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/go", &|| {
		html!(<tangram_www_docs_getting_started_predict_go_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/node", &|| {
		html!(<tangram_www_docs_getting_started_predict_node_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/python", &|| {
		html!(<tangram_www_docs_getting_started_predict_python_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/ruby", &|| {
		html!(<tangram_www_docs_getting_started_predict_ruby_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/predict/rust", &|| {
		html!(<tangram_www_docs_getting_started_predict_rust_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/inspect", &|| {
		html!(<tangram_www_docs_getting_started_inspect_server::Page />).render_to_string()
	});
	route_map.insert("/docs/getting_started/monitor", &|| {
		html!(<tangram_www_docs_getting_started_monitor_server::Page />).render_to_string()
	});
	route_map.insert("/docs/train/configuration", &|| {
		html!(<tangram_www_docs_train_configuration_server::Page />).render_to_string()
	});
	route_map
}

struct Context {
	route_map: RouteMap,
}

async fn request_handler(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> http::Response<hyper::Body> {
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
