use anyhow::{bail, Result};
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	path_components,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete")]
	Delete,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let (organization_id, member_id) = if let ["organizations", organization_id, "members", member_id] =
		*path_components(&request).as_slice()
	{
		(organization_id.to_owned(), member_id.to_owned())
	} else {
		bail!("unexpected path");
	};
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(&request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	};
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let member_id: Id = match member_id.parse() {
		Ok(member_id) => member_id,
		Err(_) => return Ok(bad_request()),
	};
	let response = match action {
		Action::Delete => {
			delete_member(&mut db, organization_id, member_id).await?;
			let redirect_location = if member_id == user.id {
				"/".to_owned()
			} else {
				format!("/organizations/{}/", organization_id)
			};
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, redirect_location)
				.body(hyper::Body::empty())
				.unwrap()
		}
	};
	db.commit().await?;
	Ok(response)
}

async fn delete_member(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	member_id: Id,
) -> Result<()> {
	sqlx::query(
		"
			delete from
				organizations_users
			where
				organization_id = $1
				and user_id = $2
		",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.execute(&mut *db)
	.await?;
	Ok(())
}
