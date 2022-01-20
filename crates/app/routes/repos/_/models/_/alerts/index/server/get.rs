use crate::page::{AlertsTable, AlertsTableRow, Page};
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
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
	let model_id = if let ["repos", _, "models", model_id, "alerts", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
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
	let model_layout_info = model_layout_info(&mut db, app, model_id, ModelNavItem::Alerts).await?;
	let alerts = app.get_all_alerts_for_model(&mut db, model_id).await?;
	let alerts_table = if !alerts.is_empty() {
		let rows = alerts
			.iter()
			.map(|row| {
				let last_updated: time::OffsetDateTime =
					Utc.timestamp(row.timestamp, 0).with_timezone(&timezone);
				AlertsTableRow {
					id: row.id.to_string(),
					last_updated: last_updated.to_string(),
				}
			})
			.collect();
		let alerts_table = AlertsTable { rows };
		Some(alerts_table)
	} else {
		None
	};
	let page = Page {
		alerts_table,
		model_layout_info,
	};
	app.commit_transaction(db).await?;
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
