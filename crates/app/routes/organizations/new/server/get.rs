use crate::page::Page;
use anyhow::Result;
use modelfox_app_context::Context;
use modelfox_app_layouts::app_layout::app_layout_info;
use pinwheel::prelude::*;
use std::sync::Arc;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app_layout_info = app_layout_info(&context.app).await?;
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
