use crate::page::Page;
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::{str::FromStr, sync::Arc};
use tangram_app_context::Context;
use tangram_app_core::{
	alerts::{get_monitor, AlertModelType},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let app = &context.app;
	let (model_id, monitor_id) = if let ["repos", _, "models", model_id, "monitors", monitor_id, "edit"] =
		path_components(request).as_slice()
	{
		(model_id.to_owned(), monitor_id.to_owned())
	} else {
		bail!("unexpected path");
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match app.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(&app.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_type = AlertModelType::from(model.inner());
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::Monitors).await?;
	let monitor = get_monitor(&mut db, Id::from_str(monitor_id)?).await?;
	let page = Page {
		monitor,
		monitor_id: monitor_id.to_string(),
		model_layout_info,
		model_type,
		error: None,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
