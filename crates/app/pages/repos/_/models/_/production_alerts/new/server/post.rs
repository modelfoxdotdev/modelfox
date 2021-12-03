use crate::page::Page;
use anyhow::{bail, Result};
use multer::Multipart;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	alerts::{AlertHeuristics, AlertThreshold},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	user::{authorize_user, authorize_user_for_model, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let repo_id = if let ["repos", repo_id, "models", _, "production_alerts", "new"] =
		*path_components(request).as_slice()
	{
		repo_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	let model_id = if let ["repos", _, "models", model_id, "production_alerts", "new"] =
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
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionAlerts).await?;
	let boundary = match request
		.headers()
		.get(http::header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			let page = Page {
				model_layout_info,
				error: Some(format!(
					"Failed to parse request body.\n{}:{}",
					file!(),
					line!()
				)),
			};
			let html = html(page);
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let mut cadence: Option<Vec<u8>> = None;
	let mut metric: Option<Vec<u8>> = None;
	let mut threshold: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.body_mut(), boundary);
	while let Some(field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				let page = Page {
					model_layout_info,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
				};
				let html = html(page);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		};
		let field_data = field.bytes().await?.to_vec();
		match name.as_str() {
			"cadence" => cadence = Some(field_data),
			"metric" => metric = Some(field_data),
			"threshold" => threshold = Some(field_data),
			_ => {
				let page = Page {
					model_layout_info,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
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
	// TODO - unwrap options, deal with error, convert Vec<u8> data to proper types
	let cadence = todo!();
	let metric = todo!();
	let variance = todo!();
	let alert = AlertHeuristics {
		cadence,
		thresholds: vec![AlertThreshold { metric, variance }],
	};
	let result = sqlx::query(
		"
			insert into 
		",
	)
	.execute(&mut db)
	.await;
	if result.is_err() {
		let page = Page {
			model_layout_info,
			error: Some("There was an error creating your alert.".to_owned()),
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
