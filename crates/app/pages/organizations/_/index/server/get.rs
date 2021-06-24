use crate::page::{
	DetailsSection, MembersSection, MembersTable, MembersTableRow, Page, ReposSection, ReposTable,
	ReposTableRow,
};
use anyhow::{anyhow, Result};
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	organizations::{get_organization, get_organization_user},
	path_components,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let organization_id =
		if let ["organizations", organization_id, ""] = *path_components(&request).as_slice() {
			organization_id.to_owned()
		} else {
			return Err(anyhow!("unexpected path"));
		};
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
	let app_layout_info = app_layout_info(&context).await?;
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
	let organization_user = get_organization_user(&mut db, organization_id, user.id)
		.await?
		.unwrap();
	let details = DetailsSection {
		organization_id: organization.id.to_string(),
		organization_name: organization.name.clone(),
		can_edit: organization_user.is_admin,
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
	let members_table = MembersTable {
		user_id: user.id.to_string(),
		rows,
		can_edit: organization_user.is_admin,
	};
	let members = MembersSection {
		organization_id: organization.id,
		members_table,
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
	let repos_table = if !rows.is_empty() {
		Some(ReposTable { rows })
	} else {
		None
	};
	let repos = ReposSection { repos_table };
	let page = Page {
		app_layout_info,
		details_section: details,
		id: organization_id.to_string(),
		members_section: members,
		name: organization.name,
		repos_section: repos,
		can_delete: organization_user.is_admin,
	};

	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
