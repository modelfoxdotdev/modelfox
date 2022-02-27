use anyhow::Result;
use std::sync::Arc;
use tangram_app_context::Context;
pub use tangram_app_core::{clock::Clock, options};
use tangram_app_core::{options::Options, App};
use tracing::error;

pub async fn run(options: Options) -> Result<()> {
	let host = options.host;
	let port = options.port;
	let addr = std::net::SocketAddr::new(host, port);
	let app = App::new(options).await?;
	let sunfish = sunfish::init!();
	let context = Context { app, sunfish };
	let context = Arc::new(context);
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
