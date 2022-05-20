use anyhow::Result;
use clap::*;
use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc};
use sunfish::Sunfish;
use tracing::error;
use tracing_subscriber::prelude::*;

struct Context {
	sunfish: Sunfish,
}

#[derive(Parser)]
enum Args {
	#[clap(name = "serve")]
	Serve,
	#[clap(name = "export")]
	Export(ExportArgs),
}

#[derive(Parser)]
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
			let addr = SocketAddr::new(host, port);
			let context = Context { sunfish };
			let context = Arc::new(context);
			modelfox_serve::serve(addr, context, handle).await?;
		}
		Args::Export(export_args) => {
			export(sunfish, export_args.path).await?;
		}
	}
	Ok(())
}

async fn export(sunfish: Sunfish, path: PathBuf) -> Result<()> {
	let out_dir = std::path::Path::new(env!("OUT_DIR"));
	let dist_path = std::env::current_dir()?.join(path);
	sunfish.export(out_dir, &dist_path)?;
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

fn setup_tracing() {
	let env_layer = tracing_subscriber::EnvFilter::try_from_env("MODELFOX_TRACING");
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
			let json_layer = tracing_subscriber::fmt::layer().json();
			let subscriber = tracing_subscriber::registry()
				.with(env_layer)
				.with(json_layer);
			subscriber.init();
		}
	}
}
