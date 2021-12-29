use crate::{
	common::{FeatureImportance, FeatureImportancesSection, TrainingSummarySection},
	page::{
		BinaryClassifier, BinaryClassifierMetricsSection, Inner, MulticlassClassifier,
		MulticlassClassifierClassMetrics, MulticlassClassifierMetricsSection, Page, Regressor,
		RegressorMetricsSection,
	},
};
use anyhow::{bail, Result};
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	heuristics::{
		TRAINING_IMPORTANCES_MAX_FEATURE_IMPORTANCES_TO_SHOW_IN_CHART,
		TRAINING_IMPORTANCES_MAX_FEATURE_IMPORTANCES_TO_SHOW_IN_TABLE,
	},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_finite::{Finite, FiniteF32};
use tangram_id::Id;
use tangram_zip::zip;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let model_id =
		if let ["repos", _, "models", model_id, ""] = *path_components(request).as_slice() {
			model_id.to_owned()
		} else {
			bail!("unexpected path");
		};
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
	let summary_section = compute_summary_section(model);
	let feature_importances_section = compute_feature_importances_section(model);
	let inner = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			let warning = if regressor.baseline_metrics().rmse() < regressor.test_metrics().rmse() {
				Some("Baseline RMSE is lower! Your model performs worse than if it were just guessing the mean of the target column.".into())
			} else {
				None
			};
			let losses_chart_series = match regressor.model() {
				tangram_model::RegressionModelReader::Linear(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
				tangram_model::RegressionModelReader::Tree(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
			};
			Inner::Regressor(Regressor {
				id: model_id.to_string(),
				training_metrics_section: RegressorMetricsSection {
					rmse: regressor.test_metrics().rmse(),
					baseline_rmse: regressor.baseline_metrics().rmse(),
					mse: regressor.test_metrics().mse(),
					baseline_mse: regressor.baseline_metrics().mse(),
					losses_chart_series,
				},
				training_summary_section: summary_section,
				feature_importances_section,
				warning,
			})
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let model = binary_classifier.read();
			let test_metrics = model.test_metrics();
			let baseline_metrics = model.baseline_metrics();
			let default_threshold_test_metrics = test_metrics.default_threshold();
			let default_threshold_baseline_metrics = baseline_metrics.default_threshold();
			let warning = if default_threshold_baseline_metrics.accuracy()
				> default_threshold_test_metrics.accuracy()
			{
				Some("Baseline Accuracy is higher! Your model performs worse than if it always predicted the majority class.".into())
			} else {
				None
			};
			let losses_chart_series = match model.model() {
				tangram_model::BinaryClassificationModelReader::Linear(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
				tangram_model::BinaryClassificationModelReader::Tree(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
			};
			Inner::BinaryClassifier(BinaryClassifier {
				id: model_id.to_string(),
				warning,
				training_metrics_section: BinaryClassifierMetricsSection {
					baseline_accuracy: default_threshold_baseline_metrics.accuracy(),
					auc_roc: model.test_metrics().auc_roc(),
					accuracy: default_threshold_test_metrics.accuracy(),
					precision: default_threshold_test_metrics.precision().unwrap(),
					recall: default_threshold_test_metrics.recall().unwrap(),
					losses_chart_series,
				},
				training_summary_section: summary_section,
				feature_importances_section,
			})
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let model = multiclass_classifier.read();
			let class_metrics = model
				.test_metrics()
				.class_metrics()
				.iter()
				.map(|class_metrics| MulticlassClassifierClassMetrics {
					precision: class_metrics.precision(),
					recall: class_metrics.recall(),
				})
				.collect::<Vec<_>>();
			let baseline_metrics = model.baseline_metrics();
			let test_metrics = model.test_metrics();
			let warning = if baseline_metrics.accuracy() > test_metrics.accuracy() {
				Some("Baseline Accuracy is higher! Your model performs worse than if it always predicted the majority class.".into())
			} else {
				None
			};
			let losses_chart_series = match model.model() {
				tangram_model::MulticlassClassificationModelReader::Linear(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
				tangram_model::MulticlassClassificationModelReader::Tree(model) => {
					let model = model.read();
					model.losses().map(|losses| losses.iter().collect())
				}
			};
			Inner::MulticlassClassifier(MulticlassClassifier {
				id: model_id.to_string(),
				training_metrics_section: MulticlassClassifierMetricsSection {
					accuracy: test_metrics.accuracy(),
					baseline_accuracy: baseline_metrics.accuracy(),
					class_metrics,
					classes: model.classes().iter().map(ToOwned::to_owned).collect(),
					losses_chart_series,
				},
				training_summary_section: summary_section,
				feature_importances_section,
				warning,
			})
		}
	};
	let model_layout_info =
		model_layout_info(&mut db, &app, model_id, ModelNavItem::Overview).await?;
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

fn compute_summary_section(model: tangram_model::ModelReader) -> TrainingSummarySection {
	let chosen_model_type_name = model_type_name(model);
	match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			TrainingSummarySection {
				chosen_model_type_name,
				column_count: regressor.overall_column_stats().len() + 1,
				comparison_metric_type_name: regression_comparison_type_name(
					&regressor.comparison_metric(),
				),
				train_row_count: regressor.train_row_count().to_usize().unwrap(),
				test_row_count: regressor.test_row_count().to_usize().unwrap(),
				comparison_row_count: regressor.overall_row_count().to_usize().unwrap()
					- regressor.train_row_count().to_usize().unwrap()
					- regressor.test_row_count().to_usize().unwrap(),
				overall_row_count: regressor.overall_row_count().to_usize().unwrap(),
			}
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			TrainingSummarySection {
				chosen_model_type_name,
				column_count: binary_classifier.overall_column_stats().len() + 1,
				comparison_metric_type_name: binary_classification_comparison_type_name(
					&binary_classifier.comparison_metric(),
				),
				train_row_count: binary_classifier.train_row_count().to_usize().unwrap(),
				test_row_count: binary_classifier.test_row_count().to_usize().unwrap(),
				comparison_row_count: binary_classifier.overall_row_count().to_usize().unwrap()
					- binary_classifier.train_row_count().to_usize().unwrap()
					- binary_classifier.test_row_count().to_usize().unwrap(),
				overall_row_count: binary_classifier.overall_row_count().to_usize().unwrap(),
			}
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			TrainingSummarySection {
				chosen_model_type_name,
				column_count: multiclass_classifier.overall_column_stats().len() + 1,
				comparison_metric_type_name: multiclass_classification_comparison_type_name(
					&multiclass_classifier.comparison_metric(),
				),
				train_row_count: multiclass_classifier.train_row_count().to_usize().unwrap(),
				test_row_count: multiclass_classifier.test_row_count().to_usize().unwrap(),
				comparison_row_count: multiclass_classifier
					.overall_row_count()
					.to_usize()
					.unwrap() - multiclass_classifier
					.train_row_count()
					.to_usize()
					.unwrap() - multiclass_classifier
					.test_row_count()
					.to_usize()
					.unwrap(),
				overall_row_count: multiclass_classifier
					.overall_row_count()
					.to_usize()
					.unwrap(),
			}
		}
	}
}

fn regression_comparison_type_name(
	comparison_metric: &tangram_model::RegressionComparisonMetricReader,
) -> String {
	match comparison_metric {
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

fn binary_classification_comparison_type_name(
	comparison_metric: &tangram_model::BinaryClassificationComparisonMetricReader,
) -> String {
	match comparison_metric {
		tangram_model::BinaryClassificationComparisonMetricReader::Aucroc(_) => {
			"Area Under the Receiver Operating Characteristic Curve".to_owned()
		}
	}
}

fn multiclass_classification_comparison_type_name(
	comparison_metric: &tangram_model::MulticlassClassificationComparisonMetricReader,
) -> String {
	match comparison_metric {
		tangram_model::MulticlassClassificationComparisonMetricReader::Accuracy(_) => {
			"Accuracy".to_owned()
		}
	}
}

fn model_type_name(model: tangram_model::ModelReader) -> String {
	match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => match regressor.read().model() {
			tangram_model::RegressionModelReader::Linear(_) => "Linear Regressor".to_owned(),
			tangram_model::RegressionModelReader::Tree(_) => {
				"Gradient Boosted Tree Regressor".to_owned()
			}
		},
		tangram_model::ModelInnerReader::BinaryClassifier(model) => match model.read().model() {
			tangram_model::BinaryClassificationModelReader::Linear(_) => {
				"Linear Binary Classifier".to_owned()
			}
			tangram_model::BinaryClassificationModelReader::Tree(_) => {
				"Gradient Boosted Tree Binary Classifier".to_owned()
			}
		},
		tangram_model::ModelInnerReader::MulticlassClassifier(model) => {
			match model.read().model() {
				tangram_model::MulticlassClassificationModelReader::Linear(_) => {
					"Linear Multiclass Classifier".to_owned()
				}
				tangram_model::MulticlassClassificationModelReader::Tree(_) => {
					"Gradient Boosted Tree Multiclass Classifier".to_owned()
				}
			}
		}
	}
}

fn compute_feature_importances_section(
	model: tangram_model::ModelReader,
) -> Option<FeatureImportancesSection> {
	let n_columns = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			regressor.read().overall_column_stats().len()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			binary_classifier.read().overall_column_stats().len()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read().overall_column_stats().len()
		}
	};
	let (mut feature_importances, n_features) = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => match regressor.read().model() {
			tangram_model::RegressionModelReader::Linear(inner_model) => {
				let inner_model = inner_model.read();
				let feature_names = compute_feature_names(inner_model.feature_groups().iter());
				let feature_importance_values = inner_model
					.feature_importances()
					.iter()
					.map(|value| Finite::new(value).ok())
					.collect::<Option<Vec<FiniteF32>>>()?;
				let mut feature_importances = zip!(feature_names, feature_importance_values)
					.map(
						|(feature_name, feature_importance_value)| FeatureImportance {
							feature_importance_value,
							feature_name,
						},
					)
					.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				(feature_importances, n_features)
			}
			tangram_model::RegressionModelReader::Tree(inner_model) => {
				let inner_model = inner_model.read();
				let feature_names = compute_feature_names(inner_model.feature_groups().iter());
				let feature_importance_values = inner_model
					.feature_importances()
					.iter()
					.map(|value| Finite::new(value).ok())
					.collect::<Option<Vec<FiniteF32>>>()?;
				let mut feature_importances = zip!(feature_names, feature_importance_values)
					.map(
						|(feature_name, feature_importance_value)| FeatureImportance {
							feature_importance_value,
							feature_name,
						},
					)
					.collect::<Vec<_>>();
				feature_importances.sort_by(|a, b| {
					a.feature_importance_value
						.partial_cmp(&b.feature_importance_value)
						.unwrap()
						.reverse()
				});
				let n_features = feature_importances.len();
				(feature_importances, n_features)
			}
		},
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			match binary_classifier.read().model() {
				tangram_model::BinaryClassificationModelReader::Linear(inner_model) => {
					let inner_model = inner_model.read();
					let feature_names = compute_feature_names(inner_model.feature_groups().iter());
					let feature_importance_values = inner_model
						.feature_importances()
						.iter()
						.map(|value| Finite::new(value).ok())
						.collect::<Option<Vec<FiniteF32>>>()?;
					let mut feature_importances = zip!(feature_names, feature_importance_values)
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_importance_value,
								feature_name,
							},
						)
						.collect::<Vec<_>>();
					feature_importances.sort_by(|a, b| {
						a.feature_importance_value
							.partial_cmp(&b.feature_importance_value)
							.unwrap()
							.reverse()
					});
					let n_features = feature_importances.len();
					(feature_importances, n_features)
				}
				tangram_model::BinaryClassificationModelReader::Tree(inner_model) => {
					let inner_model = inner_model.read();
					let feature_names = compute_feature_names(inner_model.feature_groups().iter());
					let feature_importance_values = inner_model
						.feature_importances()
						.iter()
						.map(|value| Finite::new(value).ok())
						.collect::<Option<Vec<FiniteF32>>>()?;
					let mut feature_importances = zip!(feature_names, feature_importance_values)
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_importance_value,
								feature_name,
							},
						)
						.collect::<Vec<_>>();
					feature_importances.sort_by(|a, b| {
						a.feature_importance_value
							.partial_cmp(&b.feature_importance_value)
							.unwrap()
							.reverse()
					});
					let n_features = feature_importances.len();
					(feature_importances, n_features)
				}
			}
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			match multiclass_classifier.read().model() {
				tangram_model::MulticlassClassificationModelReader::Linear(inner_model) => {
					let inner_model = inner_model.read();
					let feature_names = compute_feature_names(inner_model.feature_groups().iter());
					let feature_importance_values = inner_model
						.feature_importances()
						.iter()
						.map(|value| Finite::new(value).ok())
						.collect::<Option<Vec<FiniteF32>>>()?;
					let mut feature_importances = zip!(feature_names, feature_importance_values)
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_importance_value,
								feature_name,
							},
						)
						.collect::<Vec<_>>();
					feature_importances.sort_by(|a, b| {
						a.feature_importance_value
							.partial_cmp(&b.feature_importance_value)
							.unwrap()
							.reverse()
					});
					let n_features = feature_importances.len();
					(feature_importances, n_features)
				}
				tangram_model::MulticlassClassificationModelReader::Tree(inner_model) => {
					let inner_model = inner_model.read();
					let feature_names = compute_feature_names(inner_model.feature_groups().iter());
					let feature_importance_values = inner_model
						.feature_importances()
						.iter()
						.map(|value| Finite::new(value).ok())
						.collect::<Option<Vec<FiniteF32>>>()?;
					let mut feature_importances = zip!(feature_names, feature_importance_values)
						.map(
							|(feature_name, feature_importance_value)| FeatureImportance {
								feature_importance_value,
								feature_name,
							},
						)
						.collect::<Vec<_>>();
					feature_importances.sort_by(|a, b| {
						a.feature_importance_value
							.partial_cmp(&b.feature_importance_value)
							.unwrap()
							.reverse()
					});
					let n_features = feature_importances.len();
					(feature_importances, n_features)
				}
			}
		}
	};
	let feature_importances_table_rows = feature_importances
		.iter()
		.take(TRAINING_IMPORTANCES_MAX_FEATURE_IMPORTANCES_TO_SHOW_IN_TABLE)
		.map(|feature_importance| FeatureImportance {
			feature_name: feature_importance.feature_name.to_owned(),
			feature_importance_value: feature_importance.feature_importance_value,
		})
		.collect();
	feature_importances.truncate(TRAINING_IMPORTANCES_MAX_FEATURE_IMPORTANCES_TO_SHOW_IN_CHART);
	let feature_importances_chart_values = feature_importances;
	Some(FeatureImportancesSection {
		n_columns,
		n_features,
		feature_importances_chart_values,
		feature_importances_table_rows,
	})
}

fn compute_feature_names<'a>(
	feature_groups: impl Iterator<Item = tangram_model::FeatureGroupReader<'a>>,
) -> Vec<String> {
	feature_groups
		.flat_map(|feature_group| match feature_group {
			tangram_model::FeatureGroupReader::Identity(feature_group) => {
				let feature_group = feature_group.read();
				vec![feature_group.source_column_name().to_owned()]
			}
			tangram_model::FeatureGroupReader::Normalized(feature_group) => {
				let feature_group = feature_group.read();
				vec![feature_group.source_column_name().to_owned()]
			}
			tangram_model::FeatureGroupReader::OneHotEncoded(feature_group) => {
				let feature_group = feature_group.read();
				vec!["OOV"]
					.into_iter()
					.chain(feature_group.variants().iter())
					.map(|variant| format!("{} = {}", feature_group.source_column_name(), variant,))
					.collect()
			}
			tangram_model::FeatureGroupReader::BagOfWords(feature_group) => {
				let feature_group = feature_group.read();
				feature_group
					.ngrams()
					.iter()
					.map(|(ngram, _)| {
						format!("{} contains {}", feature_group.source_column_name(), ngram)
					})
					.collect()
			}
			tangram_model::FeatureGroupReader::BagOfWordsCosineSimilarity(feature_group) => {
				let feature_group = feature_group.read();
				vec![format!(
					"similarity of {} and {}",
					feature_group.source_column_name_a(),
					feature_group.source_column_name_b(),
				)]
			}
			tangram_model::FeatureGroupReader::WordEmbedding(feature_group) => {
				let feature_group = feature_group.read();
				(0..feature_group.model().size())
					.map(|i| {
						format!(
							"{} word embedding value {}",
							feature_group.source_column_name(),
							i
						)
					})
					.collect()
			}
		})
		.collect()
}
