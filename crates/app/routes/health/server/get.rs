use anyhow::Result;
use modelfox_app_context::Context;
use std::sync::Arc;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	match context.app.db_acquire().await {
		Ok(_) => {
			let response = http::Response::builder()
				.status(http::StatusCode::OK)
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
		Err(_) => {
			let response = http::Response::builder()
				.status(http::StatusCode::SERVICE_UNAVAILABLE)
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
	}
}
