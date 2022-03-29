use anyhow::{bail, Result};
use std::sync::Arc;
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	organizations::delete_organization,
	path_components,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
};
use modelfox_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_organization")]
	DeleteOrganization,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let organization_id =
		if let ["organizations", organization_id, ""] = *path_components(request).as_slice() {
			organization_id.to_owned()
		} else {
			bail!("unexpected path");
		};
	if !app.options().auth_enabled() {
		return Ok(not_found());
	}
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	}
	let response = match action {
		Action::DeleteOrganization => {
			delete_organization(&mut db, organization_id).await?;
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, "/user")
				.body(hyper::Body::empty())
				.unwrap()
		}
	};
	app.commit_transaction(db).await?;
	Ok(response)
}
