use crate::page::{IntervalBoxChartDataPoint, OverallBoxChartData};
use modelfox_app_date_window::{DateWindow, DateWindowInterval};
use modelfox_app_ui::{
	colors::{PRODUCTION_COLOR, TRAINING_COLOR},
	metrics_row::MetricsRow,
	time::{interval_chart_title, overall_chart_title},
};
use modelfox_charts::box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue};
use modelfox_charts::components::BoxChart;
use modelfox_ui as ui;
use num::ToPrimitive;
use pinwheel::prelude::*;

pub struct NumberColumn {
	pub column_name: String,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub alert: Option<String>,
	pub number_column_counts_section: NumberColumnCountsSection,
	pub interval_box_chart_data: Vec<IntervalBoxChartDataPoint>,
	pub overall_box_chart_data: OverallBoxChartData,
	pub number_column_stats_section: NumberColumnStatsSection,
}

pub struct NumberTrainingProductionComparison {
	pub production: Option<f32>,
	pub training: f32,
}

impl Component for NumberColumn {
	fn into_node(self) -> Node {
		let stats_overall_chart_title = overall_chart_title(&self.date_window, "Stats".to_owned());
		let overall_box_chart_series = vec![
			BoxChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: vec![BoxChartPoint {
					label: "Training".to_owned(),
					x: 0.0,
					y: Some(BoxChartValue {
						max: self.overall_box_chart_data.training.max.to_f64().unwrap(),
						min: self.overall_box_chart_data.training.min.to_f64().unwrap(),
						p25: self.overall_box_chart_data.training.p25.to_f64().unwrap(),
						p50: self.overall_box_chart_data.training.p50.to_f64().unwrap(),
						p75: self.overall_box_chart_data.training.p75.to_f64().unwrap(),
					}),
				}],
				title: Some(format!("Training Stats for {}", self.column_name)),
			},
			BoxChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: vec![BoxChartPoint {
					label: "Production".to_owned(),
					x: 0.0,
					y: self
						.overall_box_chart_data
						.production
						.map(|production| BoxChartValue {
							max: production.max.to_f64().unwrap(),
							min: production.min.to_f64().unwrap(),
							p25: production.p25.to_f64().unwrap(),
							p50: production.p50.to_f64().unwrap(),
							p75: production.p75.to_f64().unwrap(),
						}),
				}],
				title: Some(format!("Production Stats for {}", self.column_name)),
			},
		];
		let stats_interval_chart_title =
			interval_chart_title(&self.date_window_interval, "Stats".to_owned());
		let interval_box_chart_series = vec![BoxChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: self
				.interval_box_chart_data
				.iter()
				.enumerate()
				.map(|(index, entry)| BoxChartPoint {
					label: entry.label.to_owned(),
					x: index.to_f64().unwrap(),
					y: entry.stats.as_ref().map(|stats| BoxChartValue {
						max: stats.max.to_f64().unwrap(),
						min: stats.min.to_f64().unwrap(),
						p25: stats.p25.to_f64().unwrap(),
						p50: stats.p50.to_f64().unwrap(),
						p75: stats.p75.to_f64().unwrap(),
					}),
				})
				.collect(),
			title: Some(format!("Production Stats for {}", self.column_name)),
		}];
		fragment()
			.child(
				self.alert
					.map(|alert| ui::Alert::new(ui::Level::Danger).child(alert)),
			)
			.child(self.number_column_counts_section)
			.child(
				ui::Card::new().child(Dehydrate::new(
					"number_overall",
					BoxChart::new()
						.series(overall_box_chart_series)
						.title(stats_overall_chart_title),
				)),
			)
			.child(
				ui::Card::new().child(Dehydrate::new(
					"number_intervals",
					BoxChart::new()
						.series(interval_box_chart_series)
						.title(stats_interval_chart_title),
				)),
			)
			.child(self.number_column_stats_section)
			.into_node()
	}
}

pub struct NumberColumnCountsSection {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

impl Component for NumberColumnCountsSection {
	fn into_node(self) -> Node {
		MetricsRow::new()
			.child(ui::NumberCard::new(
				"Row Count".to_owned(),
				self.row_count.to_string(),
			))
			.child(ui::NumberCard::new(
				"Absent Count".to_owned(),
				self.absent_count.to_string(),
			))
			.child(ui::NumberCard::new(
				"Invalid Count".to_owned(),
				self.invalid_count.to_string(),
			))
			.into_node()
	}
}

pub struct NumberColumnStatsSection {
	pub max_comparison: NumberTrainingProductionComparison,
	pub mean_comparison: NumberTrainingProductionComparison,
	pub min_comparison: NumberTrainingProductionComparison,
	pub std_comparison: NumberTrainingProductionComparison,
}

impl Component for NumberColumnStatsSection {
	fn into_node(self) -> Node {
		fragment()
			.child(
				ui::S2::new()
					.child(
						MetricsRow::new()
							.child(
								ui::NumberComparisonCard::new(
									Some(self.min_comparison.training),
									self.min_comparison.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Min".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							)
							.child(
								ui::NumberComparisonCard::new(
									Some(self.max_comparison.training),
									self.max_comparison.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Max".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							),
					)
					.child(
						MetricsRow::new()
							.child(
								ui::NumberComparisonCard::new(
									Some(self.mean_comparison.training),
									self.mean_comparison.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Mean".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							)
							.child(
								ui::NumberComparisonCard::new(
									Some(self.std_comparison.training),
									self.std_comparison.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Standard Deviation".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							),
					),
			)
			.into_node()
	}
}
