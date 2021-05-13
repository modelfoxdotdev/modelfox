use tangram_app_common::Context;
use tangram_error::Result;

pub async fn get(
	context: &Context,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
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
