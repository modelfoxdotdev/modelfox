use crate::page::{
	AuthProps, DetailsSectionProps, Inner, NoAuthProps, OrganizationsSectionProps,
	OrganizationsTableProps, OrganizationsTableRow, Page, PageProps, ReposSectionProps,
	ReposTableProps, ReposTableRow,
};
use html::html;
use sqlx::prelude::*;
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	organizations::get_organizations,
	user::{authorize_user, User},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;
use tangram_id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_props = get_app_layout_props(context).await?;
	let props = match user {
		User::Root => {
			let repos = get_root_user_repositories(&mut db).await?;
			let rows: Vec<ReposTableRow> = repos
				.into_iter()
				.map(|repo| ReposTableRow {
					id: repo.id,
					title: repo.title,
				})
				.collect();
			let repos_table_props = if !rows.is_empty() {
				Some(ReposTableProps { rows })
			} else {
				None
			};
			let repos_section_props = ReposSectionProps { repos_table_props };
			let inner = Inner::NoAuth(NoAuthProps {
				repos_section_props,
			});
			PageProps {
				app_layout_props,
				inner,
			}
		}
		User::Normal(user) => {
			let details_section_props = DetailsSectionProps { email: user.email };
			let organizations = get_organizations(&mut db, user.id).await?;
			let rows: Vec<OrganizationsTableRow> = organizations
				.into_iter()
				.map(|organization| OrganizationsTableRow {
					id: organization.id,
					name: organization.name,
				})
				.collect();
			let organizations_table_props = if !rows.is_empty() {
				Some(OrganizationsTableProps { rows })
			} else {
				None
			};
			let organizations_section_props = OrganizationsSectionProps {
				organizations_table_props,
			};
			let repos = get_normal_user_repositories(&mut db, user.id).await?;
			let rows: Vec<ReposTableRow> = repos
				.into_iter()
				.map(|repo| ReposTableRow {
					id: repo.id,
					title: repo.title,
				})
				.collect();
			let repos_table_props = if !rows.is_empty() {
				Some(ReposTableProps { rows })
			} else {
				None
			};
			let repos_section_props = ReposSectionProps { repos_table_props };
			let inner = Inner::Auth(AuthProps {
				details_section_props,
				organizations_section_props,
				repos_section_props,
			});
			PageProps {
				app_layout_props,
				inner,
			}
		}
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

struct Repo {
	id: String,
	title: String,
}

async fn get_normal_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.user_id = $1
		",
	)
	.bind(&user_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect())
}

async fn get_root_user_repositories(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
		",
	)
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let title: String = row.get(1);
			Repo { id, title }
		})
		.collect())
}
