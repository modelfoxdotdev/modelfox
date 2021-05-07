use crate::{
	common::{ColumnStatsTableProps, ColumnStatsTableRow},
	page::{
		BinaryClassifierProps, ClassifierChartEntry, InnerProps, MulticlassClassifierProps, Page,
		PageProps, PredictionCountChartEntry, ProductionTrainingHistogram,
		ProductionTrainingQuantiles, Quantiles, RegressorChartEntry, RegressorProps,
	},
};
use chrono_tz::Tz;
use html::html;
use num::ToPrimitive;
use std::collections::BTreeMap;
use tangram_app_common::{
	column_type::ColumnType,
	date_window::DateWindow,
	date_window::{get_date_window_and_interval, DateWindowInterval},
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	heuristics::{
		PRODUCTION_STATS_LARGE_ABSENT_RATIO_THRESHOLD_TO_TRIGGER_ALERT,
		PRODUCTION_STATS_LARGE_INVALID_RATIO_THRESHOLD_TO_TRIGGER_ALERT,
	},
	model::get_model_bytes,
	production_stats::get_production_stats,
	production_stats::{
		GetProductionStatsOutput, ProductionColumnStatsOutput, ProductionPredictionStatsOutput,
		RegressionProductionPredictionStatsOutput,
	},
	time::{format_date_window, format_date_window_interval},
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{get_model_layout_props, ModelNavItem};
use tangram_error::Result;
use tangram_id::Id;

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
		ProductionPredictionStatsOutput::Regression(_) => {
			InnerProps::Regressor(compute_regressor_props(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		ProductionPredictionStatsOutput::BinaryClassification(_) => {
			InnerProps::BinaryClassifier(compute_binary_classifier_props(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
				search_params,
			))
		}
		ProductionPredictionStatsOutput::MulticlassClassification(_) => {
			InnerProps::MulticlassClassifier(compute_multiclass_classifier_props(
				model,
				production_stats,
				date_window,
				date_window_interval,
				timezone,
				search_params,
			))
		}
	};
	let model_layout_props =
		get_model_layout_props(&mut db, context, model_id, ModelNavItem::ProductionStats).await?;
	let props = PageProps {
		model_id: model_id.to_string(),
		model_layout_props,
		inner,
	};
	let html = html!(<Page {props} />).render_to_string();
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

fn compute_regressor_props(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> RegressorProps {
	let model = model.inner().as_regressor().unwrap();
	let target_column_stats = model.overall_target_column_stats();
	let overall_column_stats_table_props = compute_overall_column_stats_table_props(
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
	RegressorProps {
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table_props,
	}
}

fn compute_binary_classifier_props(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
	_search_params: Option<BTreeMap<String, String>>,
) -> BinaryClassifierProps {
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
	let overall_column_stats_table_props = compute_overall_column_stats_table_props(
		production_stats.overall.column_stats,
		production_stats.overall.row_count,
	);
	BinaryClassifierProps {
		date_window,
		date_window_interval,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table_props,
	}
}

fn compute_multiclass_classifier_props(
	model: tangram_model::ModelReader,
	production_stats: GetProductionStatsOutput,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
	search_params: Option<BTreeMap<String, String>>,
) -> MulticlassClassifierProps {
	let model = model.inner().as_multiclass_classifier().unwrap();
	let classes = model.classes().to_owned();
	let class = search_params.and_then(|s| s.get("class").map(|class| class.to_owned()));
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
	let overall_column_stats_table_props = compute_overall_column_stats_table_props(
		production_stats.overall.column_stats,
		production_stats.overall.row_count,
	);
	let classes = model
		.classes()
		.iter()
		.map(|class| class.to_owned())
		.collect::<Vec<_>>();
	MulticlassClassifierProps {
		date_window,
		date_window_interval,
		class: selected_class,
		classes,
		prediction_count_chart,
		prediction_stats_chart,
		prediction_stats_interval_chart,
		overall_column_stats_table_props,
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

fn compute_overall_column_stats_table_props(
	overall_production_column_stats: Vec<ProductionColumnStatsOutput>,
	overall_production_stats_row_count: u64,
) -> ColumnStatsTableProps {
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
	ColumnStatsTableProps { rows }
}
