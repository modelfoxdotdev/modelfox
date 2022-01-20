use crate::page::{ModelsTable, ModelsTableRow, Page};
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{not_found, redirect_to_login, service_unavailable},
	path_components,
	repos::get_repo,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_repo},
};
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let repo_id = if let ["repos", repo_id, ""] = *path_components(request).as_slice() {
		repo_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	let timezone = get_timezone(request);
	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options().auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	};
	let repo = get_repo(&mut db, repo_id).await?;
	let app_layout_info = app_layout_info(app).await?;
	let rows = sqlx::query(
		"
			select
				models.id,
				models.tag,
				models.created_at
			from models
			where models.repo_id = $1
			order by models.created_at desc
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(&mut db)
	.await?;
	let models_table = if !rows.is_empty() {
		let rows = rows
			.iter()
			.map(|row| {
				let id: String = row.get(0);
				let id: Id = id.parse().unwrap();
				let tag: Option<String> = row.get(1);
				let created_at: i64 = row.get(2);
				let created_at: time::OffsetDateTime =
					Utc.timestamp(created_at, 0).with_timezone(&timezone);
				ModelsTableRow {
					id: id.to_string(),
					tag,
					created_at: created_at.to_string(),
				}
			})
			.collect();
		let models_table = ModelsTable { rows };
		Some(models_table)
	} else {
		None
	};
	let page = Page {
		app_layout_info,
		models_table,
		title: repo.title,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	app.commit_transaction(db).await?;
	Ok(response)
}
