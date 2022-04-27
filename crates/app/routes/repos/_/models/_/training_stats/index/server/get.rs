use crate::page::{ColumnStatsTable, ColumnStatsTableRow, Page, TargetColumnStatsTable};
use anyhow::{bail, Result};
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use modelfox_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use modelfox_app_ui::column_type::ColumnType;
use modelfox_id::Id;
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::sync::Arc;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let model_id = if let ["repos", _, "models", model_id, "training_stats", ""] =
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
	let bytes = get_model_bytes(app.storage(), model_id).await?;
	let model = modelfox_model::from_bytes(&bytes)?;
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::TrainingStats).await?;
	let page = match model.inner() {
		modelfox_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			let column_stats = regressor.overall_column_stats();
			Page {
				column_stats_table: ColumnStatsTable {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(&column_stats))
						.collect(),
				},
				model_layout_info,
				column_count: column_stats.len(),
				row_count: regressor.test_row_count().to_usize().unwrap()
					+ regressor.train_row_count().to_usize().unwrap(),
				target_column_stats_table: TargetColumnStatsTable {
					target_column_stats_table_row: build_column_stats(
						&regressor.overall_target_column_stats(),
					),
				},
			}
		}
		modelfox_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			let column_stats = binary_classifier.overall_column_stats();
			Page {
				column_stats_table: ColumnStatsTable {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(&column_stats))
						.collect(),
				},
				model_layout_info,
				column_count: column_stats.len(),
				row_count: binary_classifier.test_row_count().to_usize().unwrap()
					+ binary_classifier.train_row_count().to_usize().unwrap(),
				target_column_stats_table: TargetColumnStatsTable {
					target_column_stats_table_row: build_column_stats(
						&binary_classifier.overall_target_column_stats(),
					),
				},
			}
		}
		modelfox_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			let column_stats = multiclass_classifier.overall_column_stats();
			Page {
				column_stats_table: ColumnStatsTable {
					column_stats_table_rows: column_stats
						.iter()
						.map(|column_stats| build_column_stats(&column_stats))
						.collect(),
				},
				model_layout_info,
				row_count: multiclass_classifier.test_row_count().to_usize().unwrap()
					+ multiclass_classifier.train_row_count().to_usize().unwrap(),
				column_count: column_stats.len(),
				target_column_stats_table: TargetColumnStatsTable {
					target_column_stats_table_row: build_column_stats(
						&multiclass_classifier.overall_target_column_stats(),
					),
				},
			}
		}
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	app.commit_transaction(db).await?;
	Ok(response)
}

fn build_column_stats(column_stats: &modelfox_model::ColumnStatsReader) -> ColumnStatsTableRow {
	match column_stats {
		modelfox_model::ColumnStatsReader::UnknownColumn(column_stats) => {
			let column_stats = column_stats.read();
			ColumnStatsTableRow {
				column_type: ColumnType::Unknown,
				unique_count: None,
				invalid_count: None,
				name: column_stats.column_name().to_owned(),
				max: None,
				min: None,
				std: None,
				mean: None,
				variance: None,
			}
		}
		modelfox_model::ColumnStatsReader::NumberColumn(column_stats) => {
			let column_stats = column_stats.read();
			ColumnStatsTableRow {
				column_type: ColumnType::Number,
				unique_count: Some(column_stats.unique_count().to_usize().unwrap()),
				invalid_count: Some(column_stats.invalid_count().to_usize().unwrap()),
				name: column_stats.column_name().to_owned(),
				max: Some(column_stats.max()),
				min: Some(column_stats.min()),
				std: Some(column_stats.std()),
				mean: Some(column_stats.mean()),
				variance: Some(column_stats.variance()),
			}
		}
		modelfox_model::ColumnStatsReader::EnumColumn(column_stats) => {
			let column_stats = column_stats.read();
			ColumnStatsTableRow {
				column_type: ColumnType::Enum,
				unique_count: column_stats.unique_count().to_usize(),
				invalid_count: column_stats.invalid_count().to_usize(),
				name: column_stats.column_name().to_owned(),
				max: None,
				min: None,
				std: None,
				mean: None,
				variance: None,
			}
		}
		modelfox_model::ColumnStatsReader::TextColumn(column_stats) => {
			let column_stats = column_stats.read();
			ColumnStatsTableRow {
				column_type: ColumnType::Text,
				unique_count: None,
				invalid_count: None,
				name: column_stats.column_name().to_owned(),
				max: None,
				min: None,
				std: None,
				mean: None,
				variance: None,
			}
		}
	}
}
