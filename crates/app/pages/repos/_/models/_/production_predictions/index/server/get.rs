use crate::page::{Page, Pagination, PredictionTable, PredictionTableRow};
use anyhow::{bail, Result};
use chrono::prelude::*;
use chrono_tz::Tz;
use num::ToPrimitive;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	heuristics::PRODUCTION_PREDICTIONS_NUM_PREDICTIONS_PER_PAGE_TABLE,
	monitor_event::PredictOutput,
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let model_id = if let ["repos", _, "models", model_id, "production_predictions", ""] =
		path_components(&request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	#[derive(serde::Deserialize, Default)]
	struct SearchParams {
		after: Option<i64>,
		before: Option<i64>,
	}
	let search_params: Option<SearchParams> = if let Some(query) = request.uri().query() {
		Some(serde_urlencoded::from_str(query)?)
	} else {
		None
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
	let model_layout_info = model_layout_info(
		&mut db,
		&context,
		model_id,
		ModelNavItem::ProductionPredictions,
	)
	.await?;
	let after = search_params.as_ref().and_then(|s| s.after);
	let before = search_params.as_ref().and_then(|s| s.before);
	let rows = match (after, before) {
		(Some(after), None) => {
			let rows = sqlx::query(
				"
					select
						id,
						date,
						identifier,
						input,
						output
					from (
						select
							id,
							date,
							identifier,
							input,
							output
						from predictions
						where
								model_id = $1
						and date > $2
						order by date asc
						limit $3
					)
					order by date desc
				",
			)
			.bind(&model_id.to_string())
			.bind(after)
			.bind(PRODUCTION_PREDICTIONS_NUM_PREDICTIONS_PER_PAGE_TABLE)
			.fetch_all(&mut *db)
			.await?;
			rows
		}
		(None, Some(before)) => {
			sqlx::query(
				"
				select
					id,
					date,
					identifier,
					input,
					output
				from predictions
				where
					model_id = $1
				and date < $2
				order by date desc
				limit $3
			",
			)
			.bind(&model_id.to_string())
			.bind(before)
			.bind(PRODUCTION_PREDICTIONS_NUM_PREDICTIONS_PER_PAGE_TABLE)
			.fetch_all(&mut *db)
			.await?
		}
		(None, None) => {
			sqlx::query(
				"
					select
						id,
						date,
						identifier,
						input,
						output
					from predictions
					where
						model_id = $1
					order by date desc
					limit $2
				",
			)
			.bind(&model_id.to_string())
			.bind(PRODUCTION_PREDICTIONS_NUM_PREDICTIONS_PER_PAGE_TABLE)
			.fetch_all(&mut *db)
			.await?
		}
		_ => unreachable!(),
	};
	let first_row_timestamp = rows.first().map(|row| row.get::<i64, _>(1));
	let last_row_timestamp = rows.last().map(|row| row.get::<i64, _>(1));
	let (newer_predictions_exist, older_predictions_exist) =
		match (first_row_timestamp, last_row_timestamp) {
			(Some(first_row_timestamp), Some(last_row_timestamp)) => {
				let newer_predictions_exist: bool = sqlx::query(
					"
						select count(*) > 0
						from predictions
						where model_id = $1 and date > $2
					",
				)
				.bind(&model_id.to_string())
				.bind(first_row_timestamp)
				.fetch_one(&mut *db)
				.await?
				.get(0);
				let older_predictions_exist: bool = sqlx::query(
					"
						select count(*) > 0
						from predictions
						where model_id = $1 and date < $2
					",
				)
				.bind(&model_id.to_string())
				.bind(last_row_timestamp)
				.fetch_one(&mut *db)
				.await?
				.get(0);
				(newer_predictions_exist, older_predictions_exist)
			}
			(_, _) => (false, false),
		};
	let prediction_table_rows: Vec<PredictionTableRow> = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id = id.parse().unwrap();
			let date: i64 = row.get(1);
			let date: DateTime<Tz> = Utc.timestamp(date, 0).with_timezone(&timezone);
			let identifier: String = row.get(2);
			let output: String = row.get(4);
			let output: PredictOutput = serde_json::from_str(&output).unwrap();
			let output = match output {
				PredictOutput::Regression(output) => output.value.to_string(),
				PredictOutput::BinaryClassification(output) => output.class_name,
				PredictOutput::MulticlassClassification(output) => output.class_name,
			};
			PredictionTableRow {
				id,
				date: date.to_string(),
				identifier,
				output,
			}
		})
		.collect();
	let pagination = Pagination {
		after: if newer_predictions_exist {
			first_row_timestamp.and_then(|t| t.to_usize())
		} else {
			None
		},
		before: if older_predictions_exist {
			last_row_timestamp.and_then(|t| t.to_usize())
		} else {
			None
		},
	};
	let page = Page {
		model_layout_info,
		prediction_table: if prediction_table_rows.is_empty() {
			None
		} else {
			Some(PredictionTable {
				rows: prediction_table_rows,
			})
		},
		pagination,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
