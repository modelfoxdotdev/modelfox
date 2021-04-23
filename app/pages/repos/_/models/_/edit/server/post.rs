use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_error::Result;
use tangram_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete")]
	DeleteModel,
	#[serde(rename = "update_tag")]
	UpdateTag(UpdateTagAction),
}

#[derive(serde::Deserialize)]
struct UpdateTagAction {
	tag: String,
}

pub async fn post(
	context: &Context,
	mut request: http::Request<hyper::Body>,
	repo_id: &str,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
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
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let response = match action {
		Action::DeleteModel => {
			let model_id: Id = match model_id.parse() {
				Ok(model_id) => model_id,
				Err(_) => return Ok(bad_request()),
			};
			if !authorize_user_for_model(&mut db, &user, model_id).await? {
				return Ok(not_found());
			};
			sqlx::query(
				"
					delete from models
					where id = $1
				",
			)
			.bind(&model_id.to_string())
			.execute(&mut *db)
			.await?;
			db.commit().await?;
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, format!("/repos/{}/", repo_id))
				.body(hyper::Body::empty())
				.unwrap()
		}
		Action::UpdateTag(action) => {
			sqlx::query(
				"
					update models
						set tag = $1
					where id = $2
				",
			)
			.bind(&action.tag)
			.bind(&model_id.to_string())
			.execute(&mut *db)
			.await?;
			db.commit().await?;
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(
					http::header::LOCATION,
					format!("/repos/{}/models/{}/edit", repo_id, model_id),
				)
				.body(hyper::Body::empty())
				.unwrap()
		}
	};
	Ok(response)
}
