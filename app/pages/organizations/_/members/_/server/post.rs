use num::ToPrimitive;
use sqlx::prelude::*;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	organizations::delete_organization,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};

use tangram_error::Result;
use tangram_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete")]
	Delete,
}

pub async fn post(
	context: &Context,
	mut request: http::Request<hyper::Body>,
	organization_id: &str,
	member_id: &str,
) -> Result<http::Response<hyper::Body>> {
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
			// Delete the organzation if there are no more members.
			let member_count = get_member_count(&mut db, organization_id).await?;
			if member_count == 0 {
				delete_organization(&mut db, organization_id).await?;
			};
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

async fn get_member_count(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<usize> {
	let row = sqlx::query(
		"
			select count(*) from
				organizations_users
			where
				organization_id = $1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let member_count: i64 = row.get(0);
	let member_count = member_count.to_usize().unwrap();
	Ok(member_count)
}
