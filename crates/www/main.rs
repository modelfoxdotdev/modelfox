use anyhow::Result;
use clap::Clap;
use std::{path::PathBuf, sync::Arc, time::Duration};
use sunfish::Sunfish;
use tower::{make::Shared, ServiceBuilder};
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::{error, info, Span};
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
	setup_tracing();
	let args = Args::parse();
	let sunfish = sunfish::init!();
	match args {
		Args::Serve => {
			serve(sunfish).await?;
		}
		Args::Export(export_args) => {
			export(sunfish, export_args.path).await?;
		}
	}
	Ok(())
}

async fn serve(sunfish: Sunfish) -> Result<()> {
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
	let context_layer = AddExtensionLayer::new(Arc::new(sunfish));
	let trace_layer = TraceLayer::new_for_http()
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
		.layer(trace_layer)
		.service_fn(handle);
	let addr = std::net::SocketAddr::new(host, port);
	let server = hyper::server::Server::try_bind(&addr)?;
	server.serve(Shared::new(service)).await?;
	Ok(())
}

async fn handle(
	mut request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>, http::Error> {
	let sunfish = request.extensions().get::<Arc<Sunfish>>().unwrap().clone();
	let response = sunfish.handle(&mut request).await.unwrap_or_else(|error| {
		error!(%error);
		Some(
			http::Response::builder()
				.status(http::StatusCode::INTERNAL_SERVER_ERROR)
				.body(hyper::Body::from("internal server error"))
				.unwrap(),
		)
	});
	let response = response.unwrap_or_else(|| {
		http::Response::builder()
			.status(http::StatusCode::NOT_FOUND)
			.body(hyper::Body::from("not found"))
			.unwrap()
	});
	Ok(response)
}

async fn export(sunfish: Sunfish, path: PathBuf) -> Result<()> {
	let out_dir = std::path::Path::new(env!("OUT_DIR"));
	let dist_path = std::env::current_dir()?.join(path);
	sunfish.export(&out_dir, &dist_path)?;
	Ok(())
}

fn setup_tracing() {
	let env_layer = tracing_subscriber::EnvFilter::new("[]=info");
	let format_layer = tracing_subscriber::fmt::layer().pretty();
	let subscriber = tracing_subscriber::registry()
		.with(env_layer)
		.with(format_layer);
	subscriber.init();
}
