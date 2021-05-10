use crate::page::{Page, PageProps};
use html::html;
use num::ToPrimitive;
use sqlx::prelude::*;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;
use tangram_id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
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
	}
	let app_layout_props = get_app_layout_props(context).await?;
	let row = sqlx::query(
		"
			select
				users.id,
				users.email,
				organizations_users.is_admin
			from users
			join organizations_users
				on organizations_users.organization_id = $1
				and organizations_users.user_id = $2
			where users.id = $2
		",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let member_email = row.get(1);
	let member_count = get_member_count(&mut db, organization_id).await?;
	let is_admin = row.get(2);
	let props = PageProps {
		app_layout_props,
		member_email,
		is_admin,
		can_delete: member_count > 1,
	};
	db.commit().await?;
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
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
