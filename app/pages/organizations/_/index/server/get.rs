use crate::page::{
	DetailsSectionProps, MembersSectionProps, MembersTableProps, MembersTableRow, Page, PageProps,
	ReposSectionProps, ReposTableProps, ReposTableRow,
};
use html::html;
use sqlx::prelude::*;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	organizations::get_organization,
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
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
	let app_layout_props = get_app_layout_props(context).await?;
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
	let organization = match get_organization(organization_id, &mut db).await? {
		Some(organization) => organization,
		None => return Ok(not_found()),
	};
	let details_props = DetailsSectionProps {
		organization_id: organization.id.clone(),
		organization_name: organization.name.clone(),
	};
	let rows = organization
		.members
		.into_iter()
		.map(|member| MembersTableRow {
			id: member.id,
			email: member.email,
			is_admin: member.is_admin,
		})
		.collect();
	let members_table_props = MembersTableProps {
		user_id: user.id.to_string(),
		rows,
	};
	let members_props = MembersSectionProps {
		organization_id: organization.id,
		members_table_props,
	};
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.organization_id = $1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let rows: Vec<ReposTableRow> = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			ReposTableRow { id, title }
		})
		.collect();
	let repos_table_props = if !rows.is_empty() {
		Some(ReposTableProps { rows })
	} else {
		None
	};
	let repos_props = ReposSectionProps { repos_table_props };
	let props = PageProps {
		app_layout_props,
		details_props,
		id: organization_id.to_string(),
		members_props,
		name: organization.name,
		repos_props,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
