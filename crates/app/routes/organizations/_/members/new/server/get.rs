use crate::page::Page;
use anyhow::Result;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{error::not_found, Context};
use tangram_app_layouts::app_layout::app_layout_info;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
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
