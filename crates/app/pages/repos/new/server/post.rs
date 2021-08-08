use anyhow::Result;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	repos::{create_org_repo, create_root_repo, create_user_repo},
	user::{authorize_user, authorize_user_for_organization},
	Context,
};
use tangram_id::Id;

#[derive(serde::Deserialize)]
struct Action {
	title: String,
	owner: Option<String>,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let Action { title, owner } = action;
	let repo_id = Id::generate();
	if let Some(owner) = &owner {
		let owner_parts: Vec<&str> = owner.split(':').collect();
		let owner_type = match owner_parts.get(0) {
			Some(owner_type) => owner_type,
			None => return Ok(bad_request()),
		};
		let owner_id = match owner_parts.get(1) {
			Some(owner_id) => owner_id,
			None => return Ok(bad_request()),
		};
		let owner_id: Id = match owner_id.parse() {
			Ok(owner_id) => owner_id,
			Err(_) => return Ok(bad_request()),
		};
		match *owner_type {
			"user" => {
				create_user_repo(&mut db, owner_id, repo_id, &title).await?;
			}
			"organization" => {
				if !authorize_user_for_organization(&mut db, &user, owner_id).await? {
					return Ok(not_found());
				};
				create_org_repo(&mut db, owner_id, repo_id, title.as_str()).await?;
			}
			_ => return Ok(bad_request()),
		}
	} else {
		create_root_repo(&mut db, repo_id, title.as_str()).await?;
	};
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, format!("/repos/{}/", repo_id))
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
