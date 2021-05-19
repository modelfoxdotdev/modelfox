use crate::page::Page;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::Context;
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_error::Result;

pub async fn get(
	context: Arc<Context>,
	_request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let app_layout_info = app_layout_info(&context).await?;
	let page = Page {
		app_layout_info,
		error: None,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
