use crate::page::Page;
use anyhow::{bail, Result};
use chrono_tz::Tz;
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	heuristics::PRODUCTION_STATS_TEXT_COLUMN_MAX_TOKENS_TO_SHOW_IN_TABLE,
	model::get_model_bytes,
	path_components,
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_production_stats::{
	get_production_stats, GetProductionStatsOutput, ProductionColumnStatsOutput,
};
use tangram_app_ui::{
	date_window::{get_date_window_and_interval, DateWindow, DateWindowInterval},
	time::format_date_window_interval,
};
use tangram_id::Id;
use tangram_ui as ui;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let (model_id, column_name) = if let ["repos", _, "models", model_id, "production_stats", "columns", column_name] =
		path_components(request).as_slice()
	{
		(model_id.to_owned(), column_name.to_owned())
	} else {
		bail!("unexpected path");
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	let column_name = ui::percent_decode(column_name).to_string();
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
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::ProductionStats).await?;
	let get_production_stats_output =
		get_production_stats(&mut db, model, date_window, date_window_interval, timezone).await?;
	let overall_train_row_count = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			regressor.overall_row_count()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			binary_classifier.overall_row_count()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			multiclass_classifier.overall_row_count()
		}
	};
	let overall_column_stats = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			regressor.overall_column_stats()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			binary_classifier.overall_column_stats()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			multiclass_classifier.overall_column_stats()
		}
	};
	let train_column_stats = overall_column_stats
		.iter()
		.find(|column| column.column_name() == column_name)
		.unwrap();
	let inner = match &train_column_stats {
		tangram_model::ColumnStatsReader::NumberColumn(train_column_stats) => {
			let train_column_stats = train_column_stats.read();
			Inner::Number(number_column(
				get_production_stats_output,
				train_column_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		tangram_model::ColumnStatsReader::EnumColumn(train_column_stats) => {
			let train_column_stats = train_column_stats.read();
			Inner::Enum(enum_column(
				get_production_stats_output,
				train_column_stats,
				overall_train_row_count,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		tangram_model::ColumnStatsReader::TextColumn(train_column_stats) => {
			let train_column_stats = train_column_stats.read();
			Inner::Text(text_column(
				get_production_stats_output,
				train_column_stats,
				date_window,
				date_window_interval,
				timezone,
			))
		}
		_ => return Ok(bad_request()),
	};
	let page = Page {
		date_window,
		column_name,
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

fn number_column(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: tangram_model::NumberColumnStatsReader,
	date_window: DateWindow,
	date_window_interval: DateWindowInterval,
	timezone: Tz,
) -> NumberColumn {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name()
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Number(overall) => overall,
		_ => unreachable!(),
	};
	let overall_box_chart_data = OverallBoxChartData {
		production: overall
			.stats
			.as_ref()
			.map(|stats| OverallBoxChartDataStats {
				max: stats.max,
				min: stats.min,
				p25: stats.p25,
				p50: stats.p50,
				p75: stats.p75,
			}),
		training: OverallBoxChartDataStats {
			max: train_column_stats.max(),
			min: train_column_stats.min(),
			p25: train_column_stats.p25(),
			p50: train_column_stats.p50(),
			p75: train_column_stats.p75(),
		},
	};
	let interval_box_chart_data = get_production_stats_output
		.intervals
		.iter()
		.map(|interval| {
			let production_column_stats = interval
				.column_stats
				.iter()
				.find(|production_column_stats| {
					production_column_stats.column_name() == train_column_stats.column_name()
				})
				.unwrap();
			let production_column_stats = match production_column_stats {
				ProductionColumnStatsOutput::Number(production_column_stats) => {
					production_column_stats
				}
				_ => unreachable!(),
			};
			IntervalBoxChartDataPoint {
				label: format_date_window_interval(
					interval.start_date,
					&date_window_interval,
					timezone,
				),
				stats: production_column_stats.stats.as_ref().map(|c| {
					IntervalBoxChartDataPointStats {
						max: c.max,
						min: c.min,
						p25: c.p25,
						p50: c.p50,
						p75: c.p75,
					}
				}),
			}
		})
		.collect();
	let min_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.min),
		training: train_column_stats.min(),
	};
	let max_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.max),
		training: train_column_stats.max(),
	};
	let mean_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.mean),
		training: train_column_stats.mean(),
	};
	let std_comparison = NumberTrainingProductionComparison {
		production: overall.stats.as_ref().map(|s| s.std),
		training: train_column_stats.std(),
	};
	NumberColumn {
		column_name: train_column_stats.column_name().to_owned(),
		date_window,
		date_window_interval,
		alert: None,
		number_column_counts_section: NumberColumnCountsSection {
			absent_count: overall.absent_count,
			row_count: get_production_stats_output.overall.row_count,
			invalid_count: overall.invalid_count,
		},
		interval_box_chart_data,
		overall_box_chart_data,
		number_column_stats_section: NumberColumnStatsSection {
			max_comparison,
			mean_comparison,
			min_comparison,
			std_comparison,
		},
	}
}

fn enum_column(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: tangram_model::EnumColumnStatsReader,
	overall_train_row_count: u64,
	date_window: DateWindow,
	_date_window_interval: DateWindowInterval,
	_timezone: Tz,
) -> EnumColumn {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.into_iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name()
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Enum(overall) => overall,
		_ => unreachable!(),
	};
	let production_row_count = get_production_stats_output.overall.row_count;
	let production_histogram = overall.histogram.into_iter().collect::<BTreeMap<_, _>>();
	let overall_chart_data = train_column_stats
		.histogram()
		.iter()
		.map(|(training_enum_option, training_count)| {
			let production_count = production_histogram.get(training_enum_option).unwrap_or(&0);
			let production_fraction = if production_row_count > 0 {
				Some(production_count.to_f32().unwrap() / production_row_count.to_f32().unwrap())
			} else {
				None
			};
			(
				training_enum_option.to_owned(),
				EnumColumnOverallHistogramEntry {
					production_count: *production_count,
					training_count,
					production_fraction,
					training_fraction: training_count.to_f32().unwrap()
						/ overall_train_row_count.to_f32().unwrap(),
				},
			)
		})
		.collect::<Vec<_>>();
	let enum_unique_values_table_rows = overall_chart_data
		.iter()
		.map(|(name, histogram_entry)| EnumUniqueValuesTableRow {
			name: name.to_owned(),
			training_count: histogram_entry.training_count.to_usize().unwrap(),
			production_count: histogram_entry.production_count.to_usize().unwrap(),
			training_fraction: histogram_entry.training_fraction,
			production_fraction: histogram_entry.production_fraction,
		})
		.collect();
	let enum_invalid_values_table =
		overall
			.invalid_histogram
			.as_ref()
			.map(|invalid_histogram| EnumInvalidValuesTable {
				rows: invalid_histogram
					.iter()
					.map(|(name, count)| EnumInvalidValuesTableRow {
						name: name.to_owned(),
						count: count.to_usize().unwrap(),
						production_fraction: count.to_f32().unwrap()
							/ production_row_count.to_f32().unwrap(),
					})
					.collect(),
			});
	EnumColumn {
		alert: None,
		counts_section: EnumColumnCountsSection {
			absent_count: overall.absent_count,
			row_count: production_row_count,
			invalid_count: overall.invalid_count,
		},
		stats_section: EnumColumnStatsSection {
			overall_chart_data,
			column_name: overall.column_name,
			date_window,
		},
		unique_values_section: EnumColumnUniqueValuesSection {
			enum_unique_values_table: EnumUniqueValuesTable {
				rows: enum_unique_values_table_rows,
			},
		},
		invalid_values_section: EnumColumnInvalidValuesSection {
			enum_invalid_values_table,
		},
	}
}

fn text_column(
	get_production_stats_output: GetProductionStatsOutput,
	train_column_stats: tangram_model::TextColumnStatsReader,
	date_window: DateWindow,
	_date_window_interval: DateWindowInterval,
	_timezone: Tz,
) -> TextColumn {
	let overall = get_production_stats_output
		.overall
		.column_stats
		.iter()
		.find(|production_column_stats| {
			production_column_stats.column_name() == train_column_stats.column_name()
		})
		.unwrap();
	let overall = match overall {
		ProductionColumnStatsOutput::Text(overall) => overall,
		_ => unreachable!(),
	};
	let mut text_ngrams_table_rows = overall
		.ngrams
		.iter()
		.take(PRODUCTION_STATS_TEXT_COLUMN_MAX_TOKENS_TO_SHOW_IN_TABLE)
		.map(|(ngram, entry)| TextNGramsTableRow {
			ngram: ngram.to_string(),
			count: entry.row_count.to_usize().unwrap(),
		})
		.collect::<Vec<TextNGramsTableRow>>();
	text_ngrams_table_rows.sort_by(|a, b| a.count.cmp(&b.count).reverse());
	let ngram_row_counts = overall
		.ngrams
		.iter()
		.map(|(ngram, entry)| (ngram.to_string(), entry.row_count))
		.collect();
	TextColumn {
		alert: None,
		text_column_counts_section: TextColumnCountsSection {
			row_count: get_production_stats_output.overall.row_count,
			absent_count: overall.absent_count,
			invalid_count: overall.invalid_count,
		},
		text_ngrams_section: TextColumnTokensSection {
			text_ngrams_table: TextNGramsTable {
				rows: text_ngrams_table_rows,
			},
		},
		text_column_stats_section: TextColumnStatsSection {
			column_name: overall.column_name.to_owned(),
			date_window,
			ngram_row_counts,
		},
	}
}
