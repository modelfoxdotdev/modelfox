use crate::page::Page;
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::{str::FromStr, sync::Arc};
use tangram_app_context::Context;
use tangram_app_core::{
	alert::{AlertMethod, AlertMetric},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	monitor::{
		extract_threshold_bounds, validate_threshold_bounds, AlertModelType, MonitorCadence,
		MonitorThreshold, MonitorThresholdMode,
	},
	monitor_checker::CreateMonitorArgs,
	path_components,
	user::{authorize_user, authorize_user_for_model, authorize_user_for_repo},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

#[derive(serde::Deserialize)]
struct Action {
	cadence: String,
	email: String,
	metric: String,
	mode: String,
	threshold_lower: String,
	threshold_upper: String,
	title: String,
	webhook: String,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let app = &context.app;
	let (repo_id, model_id) = if let ["repos", repo_id, "models", model_id, "monitors", "new"] =
		*path_components(request).as_slice()
	{
		(repo_id.to_owned(), model_id.to_owned())
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
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	}
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let bytes = get_model_bytes(app.storage(), model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_type = AlertModelType::from(model.inner());
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::Monitors).await?;
	let Action {
		cadence,
		email,
		metric,
		mode,
		threshold_lower,
		threshold_upper,
		title,
		webhook,
	} = action;
	let metric = AlertMetric::from_str(&metric)?;
	// Validate metric type
	let mut methods = vec![AlertMethod::Stdout];
	if !email.is_empty() {
		methods.push(AlertMethod::Email(email.into()));
	}
	if !webhook.is_empty() {
		match webhook.try_into() {
			Ok(webhook) => methods.push(AlertMethod::Webhook(webhook)),
			Err(_) => {
				let page = Page {
					model_layout_info,
					model_type,
					error: Some("Received malformed webhook url.".to_owned()),
				};
				let html = html(page);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		}
	}
	let threshold_bounds = validate_threshold_bounds(threshold_lower, threshold_upper);
	if threshold_bounds.is_none() {
		let page = Page {
			model_layout_info,
			model_type,
			error: Some("Must provide at least one threshold bound.".to_owned()),
		};
		let html = html(page);
		let response = http::Response::builder()
			.status(http::StatusCode::BAD_REQUEST)
			.body(hyper::Body::from(html))
			.unwrap();
		return Ok(response);
	}
	let (difference_lower, difference_upper) = extract_threshold_bounds(threshold_bounds.unwrap())?;
	let threshold = MonitorThreshold {
		metric,
		mode: MonitorThresholdMode::from_str(&mode)?,
		difference_lower,
		difference_upper,
	};
	let args = CreateMonitorArgs {
		db: &mut db,
		cadence: MonitorCadence::from_str(&cadence)?,
		methods: &methods,
		model_id,
		threshold,
		title: &title,
	};
	let result = app.create_monitor(args).await;
	if result.is_err() {
		let page = Page {
			model_layout_info,
			model_type,
			error: Some(result.err().unwrap().to_string()),
		};
		let html = html(page);
		let response = http::Response::builder()
			.status(http::StatusCode::BAD_REQUEST)
			.body(hyper::Body::from(html))
			.unwrap();
		return Ok(response);
	};
	app.commit_transaction(db).await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/repos/{}/models/{}/monitors/", repo_id, model_id),
		)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
