use crate::page::{Page, PageProps};
use html::html;
use std::sync::Arc;
use tangram_app_common::{error::not_found, Context};
use tangram_error::Result;

pub async fn get(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
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
	let props = PageProps {
		code: email.is_some(),
		error: None,
		email,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
