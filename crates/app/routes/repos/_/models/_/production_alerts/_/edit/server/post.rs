use crate::page::Page;
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::{str, str::FromStr, sync::Arc};
use tangram_app_common::{
	alerts::{
		delete_alert, update_alert, AlertCadence, AlertHeuristics, AlertMethod, AlertMetric,
		AlertThreshold,
	},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "update_alert")]
	UpdateAlert(UpdateAlertAction),
	#[serde(rename = "delete")]
	Delete,
}

#[derive(serde::Deserialize)]
struct UpdateAlertAction {
	cadence: String,
	email: Option<String>,
	metric: String,
	threshold: String,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let (repo_id, model_id, alert_id) = if let ["repos", repo_id, "models", model_id, "production_alerts", alert_id, "edit"] =
		*path_components(request).as_slice()
	{
		(repo_id.to_owned(), model_id.to_owned(), alert_id.to_owned())
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
		Err(_) => {
			dbg!(data);
			return Ok(bad_request());
		}
	};
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_inner = model.inner();
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionAlerts).await?;
	match action {
		Action::Delete => {
			delete_alert(&mut db, &alert_id).await?;
			db.commit().await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(
					http::header::LOCATION,
					format!("/repos/{}/models/{}/production_alerts/", repo_id, model_id),
				)
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
		Action::UpdateAlert(ua) => {
			let UpdateAlertAction {
				cadence,
				email,
				metric,
				threshold,
			} = ua;
			let metric = AlertMetric::from_str(&metric)?;
			// Validate metric type
			let mut methods = vec![AlertMethod::Stdout];
			if let Some(e) = email {
				methods.push(AlertMethod::Email(e));
			}
			let alert = AlertHeuristics {
				cadence: AlertCadence::from_str(&cadence)?,
				methods,
				threshold: AlertThreshold {
					metric,
					variance: threshold.parse()?,
				},
			};
			match metric {
				AlertMetric::Accuracy => {
					if !matches!(
						model_inner,
						tangram_model::ModelInnerReader::BinaryClassifier(_)
							| tangram_model::ModelInnerReader::MulticlassClassifier(_)
					) {
						let page = Page {
							alert,
							alert_id,
							model_layout_info,
							error: Some("Invalid metric for model type.".to_owned()),
						};
						let html = html(page);
						let response = http::Response::builder()
							.status(http::StatusCode::BAD_REQUEST)
							.body(hyper::Body::from(html))
							.unwrap();
						return Ok(response);
					}
				}
				AlertMetric::MeanSquaredError | AlertMetric::RootMeanSquaredError => {
					if !matches!(model_inner, tangram_model::ModelInnerReader::Regressor(_)) {
						let page = Page {
							alert,
							alert_id,
							model_layout_info,
							error: Some("Invalid metric for model type.".to_owned()),
						};
						let html = html(page);
						let response = http::Response::builder()
							.status(http::StatusCode::BAD_REQUEST)
							.body(hyper::Body::from(html))
							.unwrap();
						return Ok(response);
					}
				}
			};
			let result = update_alert(&mut db, &alert, &alert_id).await;
			if result.is_err() {
				let page = Page {
					alert,
					alert_id,
					model_layout_info,
					error: Some("There was an error editing your alert.".to_owned()),
				};
				let html = html(page);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			};
			db.commit().await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(
					http::header::LOCATION,
					format!("/repos/{}/models/{}/production_alerts/", repo_id, model_id),
				)
				.body(hyper::Body::empty())
				.unwrap();
			Ok(response)
		}
	}
}
