use anyhow::{bail, Result};
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	repos::delete_repo,
	user::{authorize_user, authorize_user_for_repo},
};
use modelfox_id::Id;
use std::{borrow::BorrowMut, sync::Arc};

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "update_title")]
	UpdateTitle(UpdateTitleAction),
	#[serde(rename = "delete")]
	Delete,
}

#[derive(serde::Deserialize)]
struct UpdateTitleAction {
	title: String,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let repo_id = if let ["repos", repo_id, "edit"] = *path_components(request).as_slice() {
		repo_id.to_owned()
	} else {
		bail!("unexpected path");
	};
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
	match action {
		Action::Delete => {
			delete_repo(&mut db, app.storage(), repo_id).await?;
			app.commit_transaction(db).await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, "/")
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
		Action::UpdateTitle(action) => {
			sqlx::query(
				"
					update repos
						set title = $1
					where id = $2
				",
			)
			.bind(&action.title)
			.bind(&repo_id.to_string())
			.execute(db.borrow_mut())
			.await?;
			app.commit_transaction(db).await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, format!("/repos/{}/edit", repo_id))
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
	}
}
