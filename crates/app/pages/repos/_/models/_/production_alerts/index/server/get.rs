use crate::page::{AlertsTable, AlertsTableRow, Page};
use anyhow::{bail, Result};
use chrono::prelude::*;
use chrono_tz::Tz;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	repos::get_repo,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::{
	app_layout::app_layout_info,
	model_layout::{model_layout_info, ModelNavItem},
};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	// TODO - should some of this get abstracted?
	let timezone = get_timezone(request);
	let model_id = if let ["repos", _, "models", model_id, "production_alerts", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path - model");
	};
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, context.options.auth_enabled()).await? {
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
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionAlerts).await?;
	let rows = sqlx::query(
		"
			select
				id,
				alert,
				last_updated
			from alert_preferences
			where model_id = $1
		",
	)
	.bind(model_id.to_string())
	.fetch_all(&mut db)
	.await?;
	let alerts_table = if !rows.is_empty() {
		let rows = rows
			.iter()
			.map(|row| {
				let id: String = row.get(0);
				let id: Id = id.parse().unwrap();
				let last_updated: i64 = row.get(2);
				let last_updated: DateTime<Tz> =
					Utc.timestamp(last_updated, 0).with_timezone(&timezone);
				AlertsTableRow {
					id: id.to_string(),
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
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

/*


fn alert_message(count: u64, absent_count: u64, invalid_count: u64) -> Option<String> {
	let invalid_ratio = invalid_count.to_f32().unwrap() / count.to_f32().unwrap();
	let absent_ratio = absent_count.to_f32().unwrap() / count.to_f32().unwrap();
	if invalid_ratio > PRODUCTION_STATS_LARGE_INVALID_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
		if absent_ratio > PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
			Some("High Invalid and Absent Count".to_owned())
		} else {
			Some("High Invalid Count".to_owned())
		}
	} else if absent_ratio > PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
		Some("High Absent Count".to_owned())
	} else {
		None
	}
}

*/
