use crate::page::{Page, TrainedModel};
use anyhow::{bail, Result};
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_training_grid_common::hyperparameters_for_grid_item;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let model_id = if let ["repos", _, "models", model_id, "training_grid", ""] =
		path_components(&request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
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
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::TrainingGrid).await?;
	let comparison_metric_name = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			match regressor.read().comparison_metric() {
				tangram_model::RegressionComparisonMetricReader::MeanAbsoluteError(_) => {
					"Mean Absolute Error".to_owned()
				}
				tangram_model::RegressionComparisonMetricReader::MeanSquaredError(_) => {
					"Mean Squared Error".to_owned()
				}
				tangram_model::RegressionComparisonMetricReader::RootMeanSquaredError(_) => {
					"Root Mean Squared Error".to_owned()
				}
				tangram_model::RegressionComparisonMetricReader::R2(_) => "R2".to_owned(),
			}
		}
		tangram_model::ModelInnerReader::BinaryClassifier(model) => {
			match model.read().comparison_metric() {
				tangram_model::BinaryClassificationComparisonMetricReader::Aucroc(_) => {
					"AUC".to_owned()
				}
			}
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(model) => {
			match model.read().comparison_metric() {
				tangram_model::MulticlassClassificationComparisonMetricReader::Accuracy(_) => {
					"Accuracy".to_owned()
				}
			}
		}
	};
	let trained_models_metrics: Vec<TrainedModel> = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => regressor
			.read()
			.train_grid_item_outputs()
			.iter()
			.enumerate()
			.map(|(index, grid_item)| {
				trained_model_metrics_for_grid_item(index.to_string(), &grid_item)
			})
			.collect::<Vec<_>>(),
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => binary_classifier
			.read()
			.train_grid_item_outputs()
			.iter()
			.enumerate()
			.map(|(index, grid_item)| {
				trained_model_metrics_for_grid_item(index.to_string(), &grid_item)
			})
			.collect::<Vec<_>>(),
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier
				.read()
				.train_grid_item_outputs()
				.iter()
				.enumerate()
				.map(|(index, grid_item)| {
					trained_model_metrics_for_grid_item(index.to_string(), &grid_item)
				})
				.collect::<Vec<_>>()
		}
	};
	let best_model_metrics_index = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			regressor.read().best_grid_item_index()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			binary_classifier.read().best_grid_item_index()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read().best_grid_item_index()
		}
	};
	let best_model_metrics =
		trained_models_metrics[best_model_metrics_index.to_usize().unwrap()].clone();
	let best_model = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			regressor
				.train_grid_item_outputs()
				.get(regressor.best_grid_item_index().to_usize().unwrap())
				.unwrap()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			binary_classifier
				.train_grid_item_outputs()
				.get(binary_classifier.best_grid_item_index().to_usize().unwrap())
				.unwrap()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			multiclass_classifier
				.train_grid_item_outputs()
				.get(
					multiclass_classifier
						.best_grid_item_index()
						.to_usize()
						.unwrap(),
				)
				.unwrap()
		}
	};
	let best_model_hyperparameters = hyperparameters_for_grid_item(&best_model);
	let page = Page {
		id: model_id.to_string(),
		comparison_metric_name,
		num_models: trained_models_metrics.len(),
		trained_models_metrics,
		best_model_metrics,
		best_model_hyperparameters,
		model_layout_info,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn trained_model_metrics_for_grid_item(
	identifier: String,
	train_grid_item_output: &tangram_model::TrainGridItemOutputReader,
) -> TrainedModel {
	let model_type = match &train_grid_item_output.hyperparameters() {
		tangram_model::ModelTrainOptionsReader::Linear(_) => "Linear".into(),
		tangram_model::ModelTrainOptionsReader::Tree(_) => "Gradient Boosted Tree".into(),
	};
	let duration = Duration::from_secs_f32(train_grid_item_output.duration());
	let time = format!("{:?}", duration);
	TrainedModel {
		identifier,
		comparison_metric_value: train_grid_item_output.comparison_metric_value(),
		model_type,
		time,
	}
}
