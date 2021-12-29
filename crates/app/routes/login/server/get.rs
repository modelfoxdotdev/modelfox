use crate::page::{Page, Stage};
use anyhow::Result;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::error::not_found;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	if !app.options.auth_enabled() {
		return Ok(not_found());
	}
	#[derive(serde::Deserialize)]
	struct SearchParams {
		stage: Option<SearchParamsStage>,
		email: Option<String>,
	}
	#[derive(Clone, Copy, serde::Deserialize)]
	enum SearchParamsStage {
		#[serde(rename = "email")]
		Email,
		#[serde(rename = "code")]
		Code,
	}
	let search_params: Option<SearchParams> = if let Some(query) = request.uri().query() {
		Some(serde_urlencoded::from_str(query)?)
	} else {
		None
	};
	let email = search_params
		.as_ref()
		.and_then(|search_params| search_params.email.clone());
	let stage = search_params
		.as_ref()
		.and_then(|search_params| search_params.stage)
		.map(|stage| match stage {
			SearchParamsStage::Email => Stage::Email,
			SearchParamsStage::Code => Stage::Code,
		});
	let page = Page {
		stage,
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
