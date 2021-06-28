use crate::page::Page;
use anyhow::{bail, Result};
use chrono::prelude::*;
use chrono_tz::Tz;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let model_id =
		if let ["repos", _, "models", model_id, "edit"] = *path_components(&request).as_slice() {
			model_id.to_owned()
		} else {
			bail!("unexpected path");
		};
	let timezone = get_timezone(&request);
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
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
	let app_layout_info = app_layout_info(&context).await?;
	let row = sqlx::query(
		"
			select
				models.tag,
				models.created_at
			from models
			where models.id = $1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut db)
	.await?;
	let created_at: i64 = row.get(1);
	let created_at: DateTime<Tz> = Utc.timestamp(created_at, 0).with_timezone(&timezone);
	let created_at = created_at.to_string();
	let model_tag: Option<String> = row.get(0);
	let model_heading = model_tag.clone().unwrap_or_else(|| model_id.to_string());
	let page = Page {
		app_layout_info,
		model_id,
		model_heading,
		tag: model_tag,
		created_at,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
