use crate::{
	common::{ColumnStatsTable, ColumnStatsTableRow},
	page::{
		BinaryClassifier, ClassifierChartEntry, Inner, MulticlassClassifier, Page,
		PredictionCountChartEntry, ProductionTrainingHistogram, ProductionTrainingQuantiles,
		Quantiles, Regressor, RegressorChartEntry,
	},
};
use anyhow::{bail, Result};
use chrono_tz::Tz;
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	heuristics::{
		PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT,
		PRODUCTION_STATS_LARGE_INVALID_RATIO_THRESHOLD_TO_TRIGGER_ALERT,
	},
	model::get_model_bytes,
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_production_stats::{
	get_production_stats, GetProductionStatsOutput, ProductionColumnStatsOutput,
	ProductionPredictionStatsOutput, RegressionProductionPredictionStatsOutput,
};
use tangram_app_ui::time::{format_date_window, format_date_window_interval};
use tangram_app_ui::{
	column_type::ColumnType,
	date_window::DateWindow,
	date_window::{get_date_window_and_interval, DateWindowInterval},
};
use tangram_id::Id;

#[derive(serde::Deserialize, Default)]
struct SearchParams {
	date_window: Option<DateWindow>,
	class: Option<String>,
}

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let model_id = if let ["repos", _, "models", model_id, "production_stats", ""] =
		path_components(&request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
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
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let production_stats =
		get_production_stats(&mut db, model, date_window, date_window_interval, timezone).await?;
	let inner = match production_stats.overall.prediction_stats {
		ProductionPredictionStatsOutput::Regression(_) => Inner::Regressor(compute_regressor(
			model,
			production_stats,
			date_window,
			date_window_interval,
			timezone,
		)),
		ProductionPredictionStatsOutput::BinaryClassification(_) => {
			Inner::BinaryClassifier(compute_binary_classifier(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		ProductionPredictionStatsOutput::MulticlassClassification(_) => {
			Inner::MulticlassClassifier(compute_multiclass_classifier(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
				search_params,
			))
		}
	};
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionStats).await?;
	let page = Page {
		model_id: model_id.to_string(),
		model_layout_info,
		inner,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn compute_production_training_quantiles(
	target_column_stats: &tangram_model::NumberColumnStatsReader,
	prediction_stats: &RegressionProductionPredictionStatsOutput,
) -> ProductionTrainingQuantiles {
	ProductionTrainingQuantiles {
		production: prediction_stats
			.stats
			.as_ref()
			.map(|prediction_stats| Quantiles {
				max: prediction_stats.max,
				p25: prediction_stats.p25,
				p50: prediction_stats.p50,
				p75: prediction_stats.p75,
				min: prediction_stats.min,
			}),
		training: Quantiles {
			max: target_column_stats.max(),
			min: target_column_stats.min(),
			p25: target_column_stats.p25(),
			p50: target_column_stats.p50(),
			p75: target_column_stats.p75(),
		},
	}
}

fn compute_regressor(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> Regressor {
	let model = model.inner().as_regressor().unwrap();
	let target_column_stats = model.overall_target_column_stats();
	let overall_column_stats_table = compute_overall_column_stats_table(
		production_stats.overall.column_stats,
		production_stats.overall.row_count,
	);
	let prediction_count_chart = production_stats
		.intervals
		.iter()
		.map(|interval| PredictionCountChartEntry {
			count: interval.row_count,
			label: format_date_window_interval(
				interval.start_date,
				&date_window_interval,
				timezone,
			),
		})
		.collect::<Vec<_>>();
	let target_column_stats = target_column_stats.as_number_column().unwrap();
	let prediction_stats_chart = match production_stats.overall.prediction_stats {
		ProductionPredictionStatsOutput::Regression(prediction_stats) => RegressorChartEntry {
			label: format_date_window(production_stats.overall.start_date, &date_window, timezone),
			quantiles: compute_production_training_quantiles(
				&target_column_stats,
				&prediction_stats,
			),
		},
		_ => panic!(),
	};
	let prediction_stats_interval_chart = match &production_stats.intervals[0].prediction_stats {
		ProductionPredictionStatsOutput::Regression(_) => production_stats
			.intervals
			.iter()
			.map(
				|interval_production_stats| match &interval_production_stats.prediction_stats {
					ProductionPredictionStatsOutput::Regression(interval_prediction_stats) => {
						RegressorChartEntry {
							label: format_date_window_interval(
								interval_production_stats.start_date,
								&date_window_interval,
								timezone,
							),
							quantiles: compute_production_training_quantiles(
								&target_column_stats,
								interval_prediction_stats,
							),
						}
					}
					_ => panic!(),
				},
			)
			.collect::<Vec<_>>(),
		_ => panic!(),
	};
	Regressor {
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table,
	}
}

fn compute_binary_classifier(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> BinaryClassifier {
	let model = model.inner().as_binary_classifier().unwrap();
	let target_column_stats = model.overall_target_column_stats();
	let prediction_count_chart = production_stats
		.intervals
		.iter()
		.map(|interval| PredictionCountChartEntry {
			count: interval.row_count,
			label: format_date_window_interval(
				interval.start_date,
				&date_window_interval,
				timezone,
			),
		})
		.collect::<Vec<_>>();
	let prediction_stats_chart = match production_stats.overall.prediction_stats {
		ProductionPredictionStatsOutput::BinaryClassification(prediction_stats) => {
			let target_column_stats = target_column_stats.as_enum_column().unwrap();
			let training = target_column_stats
				.histogram()
				.iter()
				.map(|(key, value)| (key.to_owned(), value))
				.collect::<Vec<_>>();
			ClassifierChartEntry {
				label: format_date_window(
					production_stats.overall.start_date,
					&date_window,
					timezone,
				),
				histogram: ProductionTrainingHistogram {
					production: prediction_stats.histogram,
					training,
				},
			}
		}
		_ => panic!(),
	};
	let prediction_stats_interval_chart = match &production_stats.intervals[0].prediction_stats {
		ProductionPredictionStatsOutput::BinaryClassification(_) => production_stats
			.intervals
			.into_iter()
			.map(
				|interval_production_stats| match interval_production_stats.prediction_stats {
					ProductionPredictionStatsOutput::BinaryClassification(prediction_stats) => {
						let target_column_stats = target_column_stats.as_enum_column().unwrap();
						let training = target_column_stats
							.histogram()
							.iter()
							.map(|(key, value)| (key.to_owned(), value))
							.collect::<Vec<_>>();
						ClassifierChartEntry {
							label: format_date_window_interval(
								interval_production_stats.start_date,
								&date_window_interval,
								timezone,
							),
							histogram: ProductionTrainingHistogram {
								production: prediction_stats.histogram,
								training,
							},
						}
					}
					_ => panic!(),
				},
			)
			.collect::<Vec<_>>(),
		_ => panic!(),
	};
	let overall_column_stats_table = compute_overall_column_stats_table(
		production_stats.overall.column_stats,
		production_stats.overall.row_count,
	);
	BinaryClassifier {
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table,
	}
}

fn compute_multiclass_classifier(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
	search_params: Option<SearchParams>,
) -> MulticlassClassifier {
	let model = model.inner().as_multiclass_classifier().unwrap();
	let class = search_params.and_then(|s| s.class);
	let classes = model.classes().to_owned();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let selected_class = class.unwrap_or_else(|| classes.get(class_index).unwrap().to_owned());
	let target_column_stats = model.overall_target_column_stats();
	let prediction_count_chart = production_stats
		.intervals
		.iter()
		.map(|interval| PredictionCountChartEntry {
			count: interval.row_count,
			label: format_date_window_interval(
				interval.start_date,
				&date_window_interval,
				timezone,
			),
		})
		.collect::<Vec<_>>();
	let start_date = production_stats.overall.start_date;
	let prediction_stats_chart = match production_stats.overall.prediction_stats {
		ProductionPredictionStatsOutput::MulticlassClassification(prediction_stats) => {
			let target_column_stats = target_column_stats.as_enum_column().unwrap();
			let training = vec![target_column_stats
				.histogram()
				.iter()
				.map(|(key, value)| (key.to_owned(), value))
				.find(|(class, _)| class == &selected_class)
				.unwrap()];
			ClassifierChartEntry {
				label: format_date_window(start_date, &date_window, timezone),
				histogram: ProductionTrainingHistogram {
					production: vec![prediction_stats
						.histogram
						.into_iter()
						.find(|(class, _)| class == &selected_class)
						.unwrap()],
					training,
				},
			}
		}
		_ => panic!(),
	};
	let interval_production_stats = production_stats.intervals;
	let prediction_stats_interval_chart = match &interval_production_stats[0].prediction_stats {
		ProductionPredictionStatsOutput::MulticlassClassification(_) => interval_production_stats
			.into_iter()
			.map(
				|interval_production_stats| match interval_production_stats.prediction_stats {
					ProductionPredictionStatsOutput::MulticlassClassification(prediction_stats) => {
						let target_column_stats = target_column_stats.as_enum_column().unwrap();
						let training = vec![target_column_stats
							.histogram()
							.iter()
							.map(|(key, value)| (key.to_owned(), value))
							.find(|(class, _)| class == &selected_class)
							.unwrap()];
						ClassifierChartEntry {
							label: format_date_window_interval(
								interval_production_stats.start_date,
								&date_window_interval,
								timezone,
							),
							histogram: ProductionTrainingHistogram {
								production: vec![prediction_stats
									.histogram
									.into_iter()
									.find(|(class, _)| class == &selected_class)
									.unwrap()],
								training,
							},
						}
					}
					_ => panic!(),
				},
			)
			.collect::<Vec<_>>(),
		_ => panic!(),
	};
	let overall_column_stats_table = compute_overall_column_stats_table(
		production_stats.overall.column_stats,
		production_stats.overall.row_count,
	);
	let classes = model
		.classes()
		.iter()
		.map(|class| class.to_owned())
		.collect::<Vec<_>>();
	MulticlassClassifier {
		date_window,
		date_window_interval,
		class: selected_class,
		classes,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table,
	}
}

fn alert_message(count: u64, absent_count: u64, invalid_count: u64) -> Option<String> {
	let invalid_ratio = invalid_count.to_f32().unwrap() / count.to_f32().unwrap();
	let absent_ratio = absent_count.to_f32().unwrap() / count.to_f32().unwrap();
	if invalid_ratio > PRODUCTION_STATS_LARGE_INVALID_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
		if absent_ratio > PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
			Some("High Invalid and Absent Count".to_owned())
		} else {
			Some("High Invalid Count".to_owned())
		}
	} else if absent_ratio > PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT {
		Some("High Absent Count".to_owned())
	} else {
		None
	}
}

fn compute_overall_column_stats_table(
	overall_production_column_stats: Vec<ProductionColumnStatsOutput>,
	overall_production_stats_row_count: u64,
) -> ColumnStatsTable {
	let rows = overall_production_column_stats
		.iter()
		.map(|column_stats| match column_stats {
			ProductionColumnStatsOutput::Unknown(column_stats) => ColumnStatsTableRow {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				href: Some(format!("./columns/{}", column_stats.column_name)),
				alert: alert_message(
					overall_production_stats_row_count,
					column_stats.absent_count,
					column_stats.invalid_count,
				),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Unknown,
			},
			ProductionColumnStatsOutput::Text(column_stats) => ColumnStatsTableRow {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				href: Some(format!("./columns/{}", column_stats.column_name)),
				alert: alert_message(
					overall_production_stats_row_count,
					column_stats.absent_count,
					column_stats.invalid_count,
				),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Text,
			},
			ProductionColumnStatsOutput::Number(column_stats) => ColumnStatsTableRow {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				href: Some(format!("./columns/{}", column_stats.column_name)),
				alert: alert_message(
					overall_production_stats_row_count,
					column_stats.absent_count,
					column_stats.invalid_count,
				),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Number,
			},
			ProductionColumnStatsOutput::Enum(column_stats) => ColumnStatsTableRow {
				absent_count: column_stats.absent_count,
				invalid_count: column_stats.invalid_count,
				href: Some(format!("./columns/{}", column_stats.column_name)),
				alert: alert_message(
					overall_production_stats_row_count,
					column_stats.absent_count,
					column_stats.invalid_count,
				),
				name: column_stats.column_name.to_owned(),
				column_type: ColumnType::Enum,
			},
		})
		.collect::<Vec<_>>();
	ColumnStatsTable { rows }
}
