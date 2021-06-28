use crate::common::{
	ColumnStatsTable, DateWindowSelectForm, PredictionCountChart, PredictionCountChartEntry,
};
use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	date_window::{DateWindow, DateWindowInterval},
	time::{interval_chart_title, overall_chart_title},
};
use tangram_charts::{
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::BoxChart,
};
use tangram_ui as ui;

pub struct Regressor {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: RegressorChartEntry,
	pub prediction_stats_interval_chart: Vec<RegressorChartEntry>,
	pub overall_column_stats_table: ColumnStatsTable,
}

impl Component for Regressor {
	fn into_node(self) -> Node {
		ui::S1::new()
			.child(ui::H1::new().child("Production Stats"))
			.child(DateWindowSelectForm {
				date_window: self.date_window,
			})
			.child(
				ui::Card::new().child(RegressionProductionStatsIntervalChart {
					chart_data: self.prediction_stats_interval_chart,
					date_window_interval: self.date_window_interval,
				}),
			)
			.child(ui::Card::new().child(PredictionCountChart {
				chart_data: self.prediction_count_chart,
				date_window_interval: self.date_window_interval,
			}))
			.child(ui::Card::new().child(RegressionProductionStatsChart {
				chart_data: self.prediction_stats_chart,
				date_window: self.date_window,
			}))
			.child(self.overall_column_stats_table)
			.into_node()
	}
}

pub struct RegressionProductionStatsChart {
	pub chart_data: RegressorChartEntry,
	pub date_window: DateWindow,
}

pub struct RegressorChartEntry {
	pub label: String,
	pub quantiles: ProductionTrainingQuantiles,
}

pub struct ProductionTrainingQuantiles {
	pub production: Option<Quantiles>,
	pub training: Quantiles,
}

pub struct Quantiles {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

impl Component for RegressionProductionStatsChart {
	fn into_node(self) -> Node {
		let series = vec![
			BoxChartSeries {
				color: ui::colors::GREEN.to_owned(),
				data: vec![BoxChartPoint {
					label: self.chart_data.label.clone(),
					x: 0.0,
					y: Some(BoxChartValue {
						max: self.chart_data.quantiles.training.max.to_f64().unwrap(),
						min: self.chart_data.quantiles.training.min.to_f64().unwrap(),
						p25: self.chart_data.quantiles.training.p25.to_f64().unwrap(),
						p50: self.chart_data.quantiles.training.p50.to_f64().unwrap(),
						p75: self.chart_data.quantiles.training.p75.to_f64().unwrap(),
					}),
				}],
				title: Some("training quantiles".to_owned()),
			},
			BoxChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: vec![BoxChartPoint {
					label: self.chart_data.label,
					x: 0.0,
					y: self
						.chart_data
						.quantiles
						.production
						.map(|production| BoxChartValue {
							max: production.max.to_f64().unwrap(),
							min: production.min.to_f64().unwrap(),
							p25: production.p25.to_f64().unwrap(),
							p50: production.p50.to_f64().unwrap(),
							p75: production.p75.to_f64().unwrap(),
						}),
				}],
				title: Some("production quantiles".to_owned()),
			},
		];
		let title = overall_chart_title(
			&self.date_window,
			"Prediction Distribution Stats".to_owned(),
		);
		Dehydrate::new(
			"quantiles_overall",
			BoxChart::new().series(series).title(title),
		)
		.into_node()
	}
}

pub struct RegressionProductionStatsIntervalChart {
	pub chart_data: Vec<RegressorChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

impl Component for RegressionProductionStatsIntervalChart {
	fn into_node(self) -> Node {
		let series = vec![BoxChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: self
				.chart_data
				.into_iter()
				.enumerate()
				.map(|(index, entry)| BoxChartPoint {
					label: entry.label,
					x: index.to_f64().unwrap(),
					y: entry.quantiles.production.map(|production| BoxChartValue {
						max: production.max.to_f64().unwrap(),
						min: production.min.to_f64().unwrap(),
						p25: production.p25.to_f64().unwrap(),
						p50: production.p50.to_f64().unwrap(),
						p75: production.p75.to_f64().unwrap(),
					}),
				})
				.collect::<Vec<_>>(),
			title: Some("quantiles".to_owned()),
		}];
		let title = interval_chart_title(
			&self.date_window_interval,
			"Prediction Distribution Stats".to_owned(),
		);
		Dehydrate::new(
			"quantiles_intervals",
			BoxChart::new().series(series).title(title),
		)
		.into_node()
	}
}
