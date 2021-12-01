use anyhow::Result;
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use sunfish::Sunfish;
use tracing::error;
use tracing_subscriber::prelude::*;

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

struct Context {
	sunfish: Sunfish,
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
	let addr = SocketAddr::new(host, port);
	let context = Context { sunfish };
	let context = Arc::new(context);
	tangram_serve::serve(addr, context, handle).await?;
	Ok(())
}

async fn handle(mut request: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
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

async fn export(sunfish: Sunfish, path: PathBuf) -> Result<()> {
	let out_dir = std::path::Path::new(env!("OUT_DIR"));
	let dist_path = std::env::current_dir()?.join(path);
	sunfish.export(out_dir, &dist_path)?;
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
