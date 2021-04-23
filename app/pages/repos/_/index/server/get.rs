use crate::page::{ModelsTableProps, ModelsTableRow, Page, PageProps};
use chrono::prelude::*;
use chrono_tz::Tz;
use html::html;
use sqlx::prelude::*;
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	repos::get_repo,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;
use tangram_id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	repo_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let timezone = get_timezone(&request);
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
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
	let app_layout_props = get_app_layout_props(context).await?;
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
	let models_table_props = if !rows.is_empty() {
		let rows = rows
			.iter()
			.map(|row| {
				let id: String = row.get(0);
				let id: Id = id.parse().unwrap();
				let tag: Option<String> = row.get(1);
				let created_at: i64 = row.get(2);
				let created_at: DateTime<Tz> =
					Utc.timestamp(created_at, 0).with_timezone(&timezone);
				ModelsTableRow {
					id: id.to_string(),
					tag,
					created_at: created_at.to_string(),
				}
			})
			.collect();
		let models_table_props = ModelsTableProps { rows };
		Some(models_table_props)
	} else {
		None
	};
	let props = PageProps {
		app_layout_props,
		models_table_props,
		title: repo.title,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
