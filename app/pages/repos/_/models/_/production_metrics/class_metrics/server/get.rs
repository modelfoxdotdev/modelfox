use crate::page::{
	ClassMetricsEntry, ConfusionMatrix, ConfusionMatrixFraction,
	ConfusionMatrixTrainingProductionComparison, InnerProps, IntervalEntry, Metrics,
	OverallClassMetrics, OverallClassMetricsEntry, Page, PageProps, TrainingProductionMetrics,
};
use html::html;
use num::ToPrimitive;
use std::collections::BTreeMap;
use tangram_app_common::{
	date_window::get_date_window_and_interval,
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	production_metrics::ProductionPredictionMetricsOutput,
	production_metrics::{get_production_metrics, GetProductionMetricsOutput},
	time::format_date_window_interval,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{get_model_layout_props, ModelNavItem};
use tangram_error::Result;
use tangram_id::Id;
use tangram_zip::zip;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<http::Response<hyper::Body>> {
	let (date_window, date_window_interval) = match get_date_window_and_interval(&search_params) {
		Some((date_window, date_window_interval)) => (date_window, date_window_interval),
		None => return Ok(bad_request()),
	};
	let timezone = get_timezone(&request);
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
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
	let bytes = get_model_bytes(&context.options.data_storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_layout_props =
		get_model_layout_props(&mut db, context, model_id, ModelNavItem::ProductionMetrics).await?;
	let production_metrics =
		get_production_metrics(&mut db, model, date_window, date_window_interval, timezone).await?;
	let model = match model.inner() {
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read()
		}
		_ => return Ok(bad_request()),
	};
	let classes: Vec<String> = model.classes().iter().map(ToOwned::to_owned).collect();
	let GetProductionMetricsOutput {
		overall, intervals, ..
	} = production_metrics;
	let overall_prediction_metrics =
		overall
			.prediction_metrics
			.map(|prediction_metrics| match prediction_metrics {
				ProductionPredictionMetricsOutput::Regression(_) => unreachable!(),
				ProductionPredictionMetricsOutput::BinaryClassification(_) => unreachable!(),
				ProductionPredictionMetricsOutput::MulticlassClassification(prediction_metrics) => {
					prediction_metrics
				}
			});
	let test_metrics = model.test_metrics();
	let training_class_metrics = test_metrics.class_metrics();
	let overall_class_metrics: Vec<OverallClassMetricsEntry> =
		zip!(training_class_metrics.iter(), classes.iter())
			.enumerate()
			.map(|(class_index, (training_class_metrics, class_name))| {
				let production_class_metrics = overall_prediction_metrics
					.as_ref()
					.map(|prediction_metrics| &prediction_metrics.class_metrics[class_index]);
				let training_total = training_class_metrics.true_negatives()
					+ training_class_metrics.true_positives()
					+ training_class_metrics.false_negatives()
					+ training_class_metrics.false_positives();
				let training_confusion_matrix_fraction = ConfusionMatrixFraction {
					false_negative_fraction: compute_fraction(
						training_class_metrics.true_negatives(),
						training_total,
					),
					false_positive_fraction: compute_fraction(
						training_class_metrics.false_positives(),
						training_total,
					),
					true_positive_fraction: compute_fraction(
						training_class_metrics.true_positives(),
						training_total,
					),
					true_negative_fraction: compute_fraction(
						training_class_metrics.true_negatives(),
						training_total,
					),
				};
				let production_confusion_matrix_fraction =
					production_class_metrics.map(|production_class_metrics| {
						let production_total = production_class_metrics.false_positives
							+ production_class_metrics.false_negatives
							+ production_class_metrics.true_positives
							+ production_class_metrics.true_negatives;
						ConfusionMatrixFraction {
							false_negative_fraction: compute_fraction(
								production_class_metrics.false_negatives,
								production_total,
							),
							true_negative_fraction: compute_fraction(
								production_class_metrics.true_negatives,
								production_total,
							),
							true_positive_fraction: compute_fraction(
								production_class_metrics.true_positives,
								production_total,
							),
							false_positive_fraction: compute_fraction(
								production_class_metrics.false_positives,
								production_total,
							),
						}
					});
				let confusion_matrix = production_class_metrics
					.map(|production_class_metrics| ConfusionMatrix {
						false_negatives: Some(
							production_class_metrics.false_negatives.to_usize().unwrap(),
						),
						true_negatives: Some(
							production_class_metrics.true_negatives.to_usize().unwrap(),
						),
						true_positives: Some(
							production_class_metrics.true_positives.to_usize().unwrap(),
						),
						false_positives: Some(
							production_class_metrics.false_negatives.to_usize().unwrap(),
						),
					})
					.unwrap_or(ConfusionMatrix {
						false_negatives: None,
						true_negatives: None,
						true_positives: None,
						false_positives: None,
					});
				let confusion_matrix_training_production_comparison =
					ConfusionMatrixTrainingProductionComparison {
						training: training_confusion_matrix_fraction,
						production: production_confusion_matrix_fraction,
					};
				OverallClassMetricsEntry {
					class_name: class_name.to_owned(),
					confusion_matrix_training_production_comparison,
					confusion_matrix,
					training: Metrics {
						f1_score: training_class_metrics.f1_score(),
						precision: training_class_metrics.precision(),
						recall: training_class_metrics.recall(),
					},
					production: production_class_metrics.map(|production_class_metrics| Metrics {
						f1_score: production_class_metrics.f1_score,
						precision: production_class_metrics.precision,
						recall: production_class_metrics.recall,
					}),
				}
			})
			.collect();
	let overall = OverallClassMetrics {
		label: format_date_window_interval(overall.start_date, &date_window_interval, timezone),
		class_metrics: overall_class_metrics,
	};
	let class_metrics: Vec<ClassMetricsEntry> = classes
		.iter()
		.enumerate()
		.map(|(class_index, class_name)| {
			let intervals = intervals
				.iter()
				.map(|interval| {
					let metrics =
						interval
							.prediction_metrics
							.as_ref()
							.map(|metrics| match metrics {
								ProductionPredictionMetricsOutput::Regression(_) => unreachable!(),
								ProductionPredictionMetricsOutput::BinaryClassification(_) => {
									unreachable!()
								}
								ProductionPredictionMetricsOutput::MulticlassClassification(
									prediction_metrics,
								) => prediction_metrics,
							});
					let production_f1_score =
						metrics.map(|m| m.class_metrics[class_index].f1_score);
					let production_recall = metrics.map(|m| m.class_metrics[class_index].recall);
					let production_precision =
						metrics.map(|m| m.class_metrics[class_index].precision);
					IntervalEntry {
						label: format_date_window_interval(
							interval.start_date,
							&date_window_interval,
							timezone,
						),
						f1_score: TrainingProductionMetrics {
							production: production_f1_score,
							training: training_class_metrics.get(class_index).unwrap().f1_score(),
						},
						precision: TrainingProductionMetrics {
							production: production_precision,
							training: training_class_metrics.get(class_index).unwrap().precision(),
						},
						recall: TrainingProductionMetrics {
							production: production_recall,
							training: training_class_metrics.get(class_index).unwrap().precision(),
						},
					}
				})
				.collect();
			ClassMetricsEntry {
				class_name: class_name.to_owned(),
				intervals,
			}
		})
		.collect();
	let class = search_params.and_then(|s| s.get("class").map(|class| class.to_owned()));
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let class = class.unwrap_or_else(|| classes.get(class_index).unwrap().to_owned());
	let props = PageProps {
		inner_props: InnerProps {
			id: model_id.to_string(),
			class_metrics,
			date_window,
			date_window_interval,
			classes,
			overall,
			class,
		},
		model_layout_props,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn compute_fraction(value: u64, total: u64) -> f32 {
	value.to_f32().unwrap() / total.to_f32().unwrap()
}
