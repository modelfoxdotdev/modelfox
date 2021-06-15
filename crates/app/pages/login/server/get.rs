use crate::page::Page;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{error::not_found, Context};
use tangram_error::Result;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
	#[derive(serde::Deserialize, Default)]
	struct SearchParams {
		email: String,
	}
	let search_params: Option<SearchParams> = if let Some(query) = request.uri().query() {
		Some(serde_urlencoded::from_str(query)?)
	} else {
		None
	};
	let email = search_params.map(|search_params| search_params.email);
	let page = Page {
		code: email.is_some(),
		error: None,
		email,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
