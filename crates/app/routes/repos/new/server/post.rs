use anyhow::Result;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	repos::{create_org_repo, create_user_repo},
	user::{authorize_user, authorize_user_for_organization},
};
use tangram_id::Id;

#[derive(serde::Deserialize)]
struct Action {
	title: String,
	owner: Option<String>,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let app_state = &app.state;
	let mut txn = match app.begin_transaction().await {
		Ok(txn) => txn,
		Err(_) => return Ok(service_unavailable())
	};
	let user = match authorize_user(request, &mut txn, app_state.options.auth_enabled()).await? {
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
	let repo_id = if let Some(owner) = &owner {
		let repo_id = Id::generate();
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
				create_user_repo(&mut txn, owner_id, repo_id, &title).await?;
				repo_id
			}
			"organization" => {
				if !authorize_user_for_organization(&mut txn, &user, owner_id).await? {
					return Ok(not_found());
				};
				create_org_repo(&mut txn, owner_id, repo_id, title.as_str()).await?;
				repo_id
			}
			_ => return Ok(bad_request()),
		}
	} else {
		app.create_root_repo(&mut txn, title.as_str()).await?
	};
	app.commit_transaction(txn).await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, format!("/repos/{}/", repo_id))
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
