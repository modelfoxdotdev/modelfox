use crate::page::Page;
use anyhow::{bail, Result};
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	monitor::AlertModelType,
	path_components,
	user::{authorize_user, authorize_user_for_model, authorize_user_for_repo},
};
use modelfox_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use modelfox_id::Id;
use pinwheel::prelude::*;
use std::sync::Arc;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let app = &context.app;
	let (repo_id, model_id) = if let ["repos", repo_id, "models", model_id, "monitors", "new"] =
		*path_components(request).as_slice()
	{
		(repo_id.to_owned(), model_id.to_owned())
	} else {
		bail!("unexpected path");
	};
	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options().auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	}
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(app.storage(), model_id).await?;
	let model = modelfox_model::from_bytes(&bytes)?;
	let model_type = AlertModelType::from(model.inner());
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::Monitors).await?;
	let page = Page {
		model_layout_info,
		model_type,
		error: None,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	app.commit_transaction(db).await?;
	Ok(response)
}
