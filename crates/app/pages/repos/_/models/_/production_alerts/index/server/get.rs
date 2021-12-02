use crate::page::Page;
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	// TODO - should some of this get abstracted?
	let model_id = if let ["repos", _, "models", model_id, "production_alerts", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
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
	//let configured_alerts = get_configured_alerts(&mut db, model_id).await?;
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionStats).await?;
	let page = Page {
		model_id: model_id.to_string(),
		model_layout_info,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

async fn get_configured_alerts(db: &sqlx::Transaction<'_, sqlx::Any>, model_id: Id) -> Result<()> {
	todo!();
}

/*
pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let model_id = if let ["repos", _, "models", model_id, "production_stats", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	let search_params: Option<SearchParams> = if let Some(query) = request.uri().query() {
		Some(serde_urlencoded::from_str(query)?)
	} else {
		None
	};
	let date_window = search_params
		.as_ref()
		.and_then(|search_params| search_params.date_window);
	let (date_window, date_window_interval) = match get_date_window_and_interval(&date_window) {
		Some((date_window, date_window_interval)) => (date_window, date_window_interval),
		None => return Ok(bad_request()),
	};
	let timezone = get_timezone(request);
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
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let production_stats =
		get_production_stats(&mut db, model, date_window, date_window_interval, timezone).await?;
	let inner = match production_stats.overall.prediction_stats {
		ProductionPredictionStatsOutput::Regression(_) => Inner::Regressor(compute_regressor(
			model,
			production_stats,
			date_window,
			date_window_interval,
			timezone,
		)),
		ProductionPredictionStatsOutput::BinaryClassification(_) => {
			Inner::BinaryClassifier(compute_binary_classifier(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		ProductionPredictionStatsOutput::MulticlassClassification(_) => {
			Inner::MulticlassClassifier(compute_multiclass_classifier(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
				search_params,
			))
		}
	};
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionStats).await?;
	let page = Page {
		model_id: model_id.to_string(),
		model_layout_info,
		inner,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}


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
