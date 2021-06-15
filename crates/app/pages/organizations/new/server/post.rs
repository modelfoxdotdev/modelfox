use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, service_unavailable, unauthorized},
	user::{authorize_normal_user, NormalUser},
	Context,
};
use tangram_error::Result;
use tangram_id::Id;

#[derive(serde::Deserialize)]
struct Action {
	name: String,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	if !context.options.auth_enabled() {
		return Ok(bad_request());
	}
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
	let user = match authorize_normal_user(&request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let response = create_organization(action, &user, &mut db).await?;
	db.commit().await?;
	Ok(response)
}

async fn create_organization(
	action: Action,
	user: &NormalUser,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<http::Response<hyper::Body>> {
	let Action { name } = action;
	let organization_id: Id = Id::generate();
	sqlx::query(
		"
			insert into organizations
				(id, name)
			values
				($1, $2)
			",
	)
	.bind(&organization_id.to_string())
	.bind(&name)
	.execute(&mut *db)
	.await?;
	sqlx::query(
		"
			insert into organizations_users
				(organization_id, user_id, is_admin)
			values
				($1, $2, true)
		",
	)
	.bind(&organization_id.to_string())
	.bind(&user.id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
