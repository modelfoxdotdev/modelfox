use crate::page::{
	AccuracyChart, AccuracyChartEntry, BinaryClassificationOverallProductionMetrics,
	BinaryClassifierProductionMetrics, ClassMetricsTableEntry, Inner, MeanSquaredErrorChart,
	MeanSquaredErrorChartEntry, MulticlassClassificationOverallProductionMetrics,
	MulticlassClassifierProductionMetrics, Page, RegressionProductionMetrics,
	RegressorProductionMetrics, TrainingProductionMetrics, TrueValuesCountChartEntry,
};
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	production_metrics::{get_production_metrics, ProductionPredictionMetricsOutput},
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_ui::{
	date_window::{get_date_window_and_interval, DateWindow},
	time::format_date_window_interval,
};
use tangram_id::Id;
use tangram_zip::zip;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let model_id = if let ["repos", _, "models", model_id, "production_metrics", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	#[derive(serde::Deserialize, Default)]
	struct SearchParams {
		date_window: Option<DateWindow>,
	}
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
	let mut db = match app.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options.auth_enabled()).await? {
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
	let bytes = get_model_bytes(&app.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let production_metrics =
		get_production_metrics(&mut db, model, date_window, date_window_interval, timezone).await?;
	let inner = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			let training_metrics = regressor.test_metrics();
			let true_values_count = production_metrics.overall.true_values_count;
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::Regression(metrics) => metrics,
						_ => unreachable!(),
					});
			let overall = RegressionProductionMetrics {
				mse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.mse),
					training: training_metrics.mse(),
				},
				rmse: TrainingProductionMetrics {
					production: overall_production_metrics.as_ref().map(|m| m.rmse),
					training: training_metrics.rmse(),
				},
				true_values_count,
			};
			let mse_chart = {
				let data = production_metrics
					.intervals
					.iter()
					.map(|interval| {
						let label = format_date_window_interval(
							interval.start_date,
							&date_window_interval,
							timezone,
						);
						let mse = interval
							.prediction_metrics
							.as_ref()
							.map(|prediction_metrics| {
								if let ProductionPredictionMetricsOutput::Regression(
									predicion_metrics,
								) = prediction_metrics
								{
									predicion_metrics.mse
								} else {
									unreachable!()
								}
							});
						MeanSquaredErrorChartEntry { label, mse }
					})
					.collect();
				MeanSquaredErrorChart {
					data,
					training_mse: training_metrics.mse(),
				}
			};
			let true_values_count_chart = production_metrics
				.intervals
				.iter()
				.map(|interval| TrueValuesCountChartEntry {
					count: interval.true_values_count,
					label: format_date_window_interval(
						interval.start_date,
						&date_window_interval,
						timezone,
					),
				})
				.collect();
			Inner::Regressor(RegressorProductionMetrics {
				date_window,
				date_window_interval,
				mse_chart,
				overall,
				true_values_count_chart,
			})
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::BinaryClassification(metrics) => metrics,
						_ => unreachable!(),
					});
			let true_values_count_chart = production_metrics
				.intervals
				.iter()
				.map(|interval| TrueValuesCountChartEntry {
					count: interval.true_values_count,
					label: format_date_window_interval(
						interval.start_date,
						&date_window_interval,
						timezone,
					),
				})
				.collect();
			let accuracy_chart = {
				let data = production_metrics
					.intervals
					.iter()
					.map(|interval| {
						let label = format_date_window_interval(
							interval.start_date,
							&date_window_interval,
							timezone,
						);
						let accuracy =
							interval
								.prediction_metrics
								.as_ref()
								.map(|prediction_metrics| {
									if let ProductionPredictionMetricsOutput::BinaryClassification(
										predicion_metrics,
									) = prediction_metrics
									{
										predicion_metrics.accuracy
									} else {
										unreachable!()
									}
								});
						AccuracyChartEntry { accuracy, label }
					})
					.collect();
				let test_metrics = binary_classifier.test_metrics();
				let default_threshold_test_metrics = test_metrics.default_threshold();
				let training_accuracy = default_threshold_test_metrics.accuracy();
				AccuracyChart {
					data,
					training_accuracy,
				}
			};
			let true_values_count = production_metrics.overall.true_values_count;
			let production_accuracy = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.accuracy);
			let production_precision = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.precision);
			let production_recall = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.recall);
			let test_metrics = binary_classifier.test_metrics();
			let default_threshold_test_metrics = test_metrics.default_threshold();
			let overall = BinaryClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: default_threshold_test_metrics.accuracy(),
				},
				precision: TrainingProductionMetrics {
					production: production_precision,
					training: default_threshold_test_metrics.precision().unwrap(),
				},
				recall: TrainingProductionMetrics {
					production: production_recall,
					training: default_threshold_test_metrics.recall().unwrap(),
				},
				true_values_count,
			};
			Inner::BinaryClassifier(BinaryClassifierProductionMetrics {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: model_id.to_string(),
				accuracy_chart,
				overall,
			})
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			let training_metrics = multiclass_classifier.test_metrics();
			let overall_production_metrics =
				production_metrics
					.overall
					.prediction_metrics
					.map(|metrics| match metrics {
						ProductionPredictionMetricsOutput::MulticlassClassification(metrics) => {
							metrics
						}
						_ => unreachable!(),
					});
			let true_values_count_chart = production_metrics
				.intervals
				.iter()
				.map(|interval| TrueValuesCountChartEntry {
					count: interval.true_values_count,
					label: format_date_window_interval(
						interval.start_date,
						&date_window_interval,
						timezone,
					),
				})
				.collect();
			let accuracy_chart = {
				let data = production_metrics
					.intervals
					.iter()
					.map(|interval| {
						let label = format_date_window_interval(
							interval.start_date,
							&date_window_interval,
							timezone,
						);
						let accuracy =
							interval
								.prediction_metrics
								.as_ref()
								.map(|prediction_metrics| {
									if let ProductionPredictionMetricsOutput::MulticlassClassification(
										predicion_metrics,
									) = prediction_metrics
									{
										predicion_metrics.accuracy
									} else {
										unreachable!()
									}
								});
						AccuracyChartEntry { accuracy, label }
					})
					.collect();
				AccuracyChart {
					data,
					training_accuracy: training_metrics.accuracy(),
				}
			};
			let true_values_count = production_metrics.overall.true_values_count;
			let training_class_metrics = training_metrics.class_metrics();
			let production_accuracy = overall_production_metrics
				.as_ref()
				.map(|metrics| metrics.accuracy);
			let production_class_metrics = overall_production_metrics
				.map(|production_metrics| production_metrics.class_metrics);
			let class_metrics_table = zip!(
				training_class_metrics.iter(),
				multiclass_classifier.classes().iter()
			)
			.enumerate()
			.map(|(class_index, (training_class_metrics, class_name))| {
				let precision = production_class_metrics
					.as_ref()
					.map(|p| p[class_index].precision);
				let recall = production_class_metrics
					.as_ref()
					.map(|p| p[class_index].recall);
				ClassMetricsTableEntry {
					precision: TrainingProductionMetrics {
						training: training_class_metrics.precision(),
						production: precision,
					},
					recall: TrainingProductionMetrics {
						training: training_class_metrics.recall(),
						production: recall,
					},
					class_name: class_name.to_owned(),
				}
			})
			.collect();
			let overall = MulticlassClassificationOverallProductionMetrics {
				accuracy: TrainingProductionMetrics {
					production: production_accuracy,
					training: training_metrics.accuracy(),
				},
				class_metrics_table_rows: class_metrics_table,
				true_values_count,
			};
			Inner::MulticlassClassifier(MulticlassClassifierProductionMetrics {
				date_window,
				date_window_interval,
				true_values_count_chart,
				id: model_id.to_string(),
				accuracy_chart,
				overall,
			})
		}
	};
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::ProductionMetrics).await?;
	let page = Page {
		id: model_id.to_string(),
		inner,
		model_layout_info,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
