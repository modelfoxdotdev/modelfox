use crate::page::{
	Auth, DetailsSection, Inner, NoAuth, OrganizationsSection, OrganizationsTable,
	OrganizationsTableRow, Page, ReposSection, ReposTable, ReposTableRow,
};
use anyhow::Result;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{not_found, redirect_to_login, service_unavailable},
	organizations::get_organizations,
	user::{authorize_user, User},
};
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	if !app.options.auth_enabled() {
		return Ok(not_found());
	}
	let mut db = match app.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_info = app_layout_info(&app).await?;
	let page = match user {
		User::Root => {
			let repos = get_root_user_repositories(&mut db).await?;
			let rows: Vec<ReposTableRow> = repos
				.into_iter()
				.map(|repo| ReposTableRow {
					id: repo.id,
					title: repo.title,
				})
				.collect();
			let repos_table = if !rows.is_empty() {
				Some(ReposTable { rows })
			} else {
				None
			};
			let repos_section = ReposSection { repos_table };
			let inner = Inner::NoAuth(NoAuth { repos_section });
			Page {
				app_layout_info,
				inner,
			}
		}
		User::Normal(user) => {
			let details_section = DetailsSection { email: user.email };
			let organizations = get_organizations(&mut db, user.id).await?;
			let rows: Vec<OrganizationsTableRow> = organizations
				.into_iter()
				.map(|organization| OrganizationsTableRow {
					id: organization.id,
					name: organization.name,
				})
				.collect();
			let organizations_table = if !rows.is_empty() {
				Some(OrganizationsTable { rows })
			} else {
				None
			};
			let organizations_section = OrganizationsSection {
				organizations_table,
			};
			let repos = get_normal_user_repositories(&mut db, user.id).await?;
			let rows: Vec<ReposTableRow> = repos
				.into_iter()
				.map(|repo| ReposTableRow {
					id: repo.id,
					title: repo.title,
				})
				.collect();
			let repos_table = if !rows.is_empty() {
				Some(ReposTable { rows })
			} else {
				None
			};
			let repos_section = ReposSection { repos_table };
			let inner = Inner::Auth(Auth {
				details_section,
				organizations_section,
				repos_section,
			});
			Page {
				app_layout_info,
				inner,
			}
		}
	};
	let html = html(page);
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
