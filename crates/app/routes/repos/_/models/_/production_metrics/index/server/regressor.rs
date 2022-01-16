use crate::page::{TrainingProductionMetrics, TrueValuesCountChartEntry};
use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	colors::{PRODUCTION_COLOR, TRAINING_COLOR},
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	metrics_row::MetricsRow,
	time::interval_chart_title,
};
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

pub struct RegressorProductionMetrics {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub mse_chart: MeanSquaredErrorChart,
	pub overall: RegressionProductionMetrics,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

pub struct MeanSquaredErrorChart {
	pub data: Vec<MeanSquaredErrorChartEntry>,
	pub training_mse: f32,
}

pub struct MeanSquaredErrorChartEntry {
	pub label: String,
	pub mse: Option<f32>,
}

pub struct RegressionProductionMetrics {
	pub mse: TrainingProductionMetrics,
	pub rmse: TrainingProductionMetrics,
	pub true_values_count: u64,
}

impl Component for RegressorProductionMetrics {
	fn into_node(self) -> Node {
		let mse_chart_labels = self
			.mse_chart
			.data
			.iter()
			.map(|entry| entry.label.clone())
			.collect::<Vec<_>>();
		let mse_series = vec![
			LineChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: (0..self.mse_chart.data.len())
					.map(|index| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: Some(
							Finite::new(self.mse_chart.training_mse.to_f64().unwrap()).unwrap(),
						),
					})
					.collect::<Vec<_>>(),
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Training Mean Squared Error".to_owned()),
			},
			LineChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.mse_chart
					.data
					.iter()
					.enumerate()
					.map(|(index, entry)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: entry
							.mse
							.map(|mse| Finite::new(mse.to_f64().unwrap()).unwrap()),
					})
					.collect::<Vec<_>>(),
				line_style: None,
				point_style: None,
				title: Some("Production Mean Squared Error".to_owned()),
			},
		];
		let mse_chart_title =
			interval_chart_title(&self.date_window_interval, "Mean Squared Error".to_owned());
		ui::S1::new()
			.child(ui::H1::new("Production Metrics"))
			.child(
				ui::S2::new()
					.child(
						ui::Form::new()
							.child(DateWindowSelectField::new(self.date_window))
							.child(
								noscript().child(
									ui::Button::new()
										.button_type(ui::ButtonType::Submit)
										.child("Submit"),
								),
							),
					)
					.child(
						ui::P::new()
							.child("You have logged ")
							.child(b().child(self.overall.true_values_count.to_string()))
							.child(" true values for this date range."),
					)
					.child(
						ui::Card::new().child(Dehydrate::new(
							"mse",
							LineChart::new()
								.labels(mse_chart_labels)
								.series(mse_series)
								.title(mse_chart_title)
								.x_axis_grid_line_interval(GridLineInterval { k: 1.0, p: 0.0 })
								.y_max(Finite::new(1.0).unwrap())
								.y_min(Finite::new(0.0).unwrap()),
						)),
					)
					.child(MetricsRow::new().child(ui::NumberCard::new(
						"True Value Count".to_owned(),
						self.overall.true_values_count.to_string(),
					)))
					.child(
						MetricsRow::new()
							.child(
								ui::NumberComparisonCard::new(
									Some(self.overall.rmse.training),
									self.overall.rmse.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Root Mean Squared Error".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							)
							.child(
								ui::NumberComparisonCard::new(
									Some(self.overall.mse.training),
									self.overall.mse.production,
								)
								.color_a(TRAINING_COLOR.to_owned())
								.color_b(PRODUCTION_COLOR.to_owned())
								.title("Mean Squared Error".to_owned())
								.value_a_title("Training".to_owned())
								.value_b_title("Production".to_owned())
								.number_formatter(ui::NumberFormatter::Float(Default::default())),
							),
					),
			)
			.into_node()
	}
}
