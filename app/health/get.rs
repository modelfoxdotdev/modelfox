use std::sync::Arc;
use tangram_app_common::Context;
use tangram_error::Result;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	match context.database_pool.acquire().await {
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
