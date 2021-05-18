use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	repos::delete_repo,
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_error::{err, Result};
use tangram_id::Id;

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

pub async fn post(
	context: Arc<Context>,
	mut request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let repo_id = if let &["repos", repo_id, "edit"] = path_components(&request).as_slice() {
		repo_id.to_owned()
	} else {
		return Err(err!("unexpected path"));
	};
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
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
			delete_repo(&mut db, &context.storage, repo_id).await?;
			db.commit().await?;
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
			.execute(&mut *db)
			.await?;
			db.commit().await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, format!("/repos/{}/edit", repo_id))
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
	}
}
