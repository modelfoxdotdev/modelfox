use anyhow::{bail, Result};
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	download_inner(request).await
}

pub async fn download_inner(
	request: &mut http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app_state = &context.app.state;
	let model_id =
		if let ["repos", _, "models", model_id, "download"] = path_components(request).as_slice() {
			model_id.to_owned()
		} else {
			bail!("unexpected path");
		};
	let mut db = match app_state.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app_state.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(&app_state.storage, model_id).await?;
	let bytes = bytes.to_owned();
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(bytes))
		.unwrap();
	Ok(response)
}
