use crate::page::{MonitorsTable, MonitorsTableRow, Page};
use anyhow::{bail, Result};
use chrono::prelude::*;
use chrono_tz::Tz;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	alerts::Monitor,
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let app = &context.app;
	let timezone = get_timezone(request);
	let model_id = if let ["repos", _, "models", model_id, "monitors", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path - model");
	};
	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options().auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::Monitors).await?;
	let rows = sqlx::query(
		"
			select
				id,
				data,
				last_checked
			from monitors
			where model_id = $1
		",
	)
	.bind(model_id.to_string())
	.fetch_all(&mut db)
	.await?;
	let monitors_table = if !rows.is_empty() {
		let rows = rows
			.iter()
			.map(|row| {
				let id: String = row.get(0);
				let id: Id = id.parse().unwrap();
				let monitor: String = row.get(1);
				let monitor: Monitor = serde_json::from_str(&monitor).unwrap();
				let name = monitor.title;
				let last_updated: i64 = row.get(2);
				let last_updated: DateTime<Tz> =
					Utc.timestamp(last_updated, 0).with_timezone(&timezone);
				MonitorsTableRow {
					id: id.to_string(),
					name,
					last_updated: last_updated.to_string(),
				}
			})
			.collect();
		let monitors_table = MonitorsTable { rows };
		Some(monitors_table)
	} else {
		None
	};
	let page = Page {
		monitors_table,
		model_layout_info,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	app.commit_transaction(db).await?;
	Ok(response)
}
