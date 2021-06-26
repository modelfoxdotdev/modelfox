use crate::page::{
	AccuracyChart, ClassMetricsTableEntry, TrainingProductionMetrics, TrueValuesCountChartEntry,
};
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

pub struct MulticlassClassifierProductionMetrics {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: MulticlassClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

pub struct MulticlassClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub class_metrics_table_rows: Vec<ClassMetricsTableEntry>,
	pub true_values_count: u64,
}

impl Component for MulticlassClassifierProductionMetrics {
	fn into_node(self) -> Node {
		let chart_labels = self
			.accuracy_chart
			.data
			.iter()
			.map(|entry| entry.label.clone())
			.collect::<Vec<_>>();
		let accuracy_series = vec![
			LineChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: (0..self.accuracy_chart.data.len())
					.map(|index| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: Some(
							Finite::new(self.accuracy_chart.training_accuracy.to_f64().unwrap())
								.unwrap(),
						),
					})
					.collect::<Vec<_>>(),
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Training Accuracy".to_owned()),
			},
			LineChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.accuracy_chart
					.data
					.iter()
					.enumerate()
					.map(|(index, entry)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: entry
							.accuracy
							.and_then(|accuracy| Finite::new(accuracy.to_f64().unwrap()).ok()),
					})
					.collect::<Vec<_>>(),
				line_style: None,
				point_style: None,
				title: Some("Production Accuracy".to_owned()),
			},
		];
		let accuracy_chart_title =
			interval_chart_title(&self.date_window_interval, "Accuracy".to_owned());
		ui::S1::new()
			.child(ui::H1::new().child("Production Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("".to_owned(), true).child("Overview"))
					.child(
						ui::TabLink::new(
							format!("class_metrics?date_window={}", self.date_window),
							false,
						)
						.child("Class Metrics"),
					),
			)
			.child(
				ui::S2::new()
					.child(
						ui::Form::new()
							.child(DateWindowSelectField::new(self.date_window))
							.child(
								noscript().child(
									ui::Button::new()
										.button_type(Some(ui::ButtonType::Submit))
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
					.child(MetricsRow::new().child(ui::NumberCard::new(
						"True Value Count".to_owned(),
						self.overall.true_values_count.to_string(),
					))),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Accuracy"))
					.child(
						ui::P::new()
							.child("Accuracy is the percentage of predictions that were correct."),
					)
					.child(
						ui::NumberComparisonCard::new(
							Some(self.overall.accuracy.training),
							self.overall.accuracy.production,
						)
						.color_a(Some(TRAINING_COLOR.to_owned()))
						.color_b(Some(PRODUCTION_COLOR.to_owned()))
						.title("Accuracy".to_owned())
						.value_a_title("Training".to_owned())
						.value_b_title("Production".to_owned())
						.number_formatter(ui::NumberFormatter::Percent(Default::default())),
					)
					.child(
						ui::Card::new().child(Dehydrate::new(
							"accuracy",
							LineChart::new()
								.labels(Some(chart_labels))
								.series(Some(accuracy_series))
								.title(Some(accuracy_chart_title))
								.x_axis_grid_line_interval(Some(GridLineInterval {
									k: 1.0,
									p: 0.0,
								}))
								.y_max(Some(Finite::new(1.0).unwrap()))
								.y_min(Some(Finite::new(0.0).unwrap())),
						)),
					),
			)
			.child(ClassMetricsTable {
				rows: self.overall.class_metrics_table_rows,
			})
			.into_node()
	}
}

pub struct ClassMetricsTable {
	rows: Vec<ClassMetricsTableEntry>,
}

impl Component for ClassMetricsTable {
	fn into_node(self) -> Node {
		let title = ui::H2::new().child("Precision and Recall");
		let p = ui::P::new().child("Precision is the percentage of examples that were labeled as this class that are actually this class. Recall is the percentage of examples that are of this class that were labeled as this class.");
		let table = ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Class"))
						.child(ui::TableHeaderCell::new().child("Training Precision"))
						.child(ui::TableHeaderCell::new().child("Training Recall"))
						.child(ui::TableHeaderCell::new().child("Production Precision"))
						.child(ui::TableHeaderCell::new().child("Production Recall")),
				),
			)
			.child(ui::TableBody::new().children(self.rows.iter().map(|c| {
				ui::TableRow::new()
					.child(ui::TableCell::new().child(c.class_name.to_owned()))
					.child(ui::TableCell::new().child(ui::format_percent(c.precision.training)))
					.child(ui::TableCell::new().child(ui::format_percent(c.recall.training)))
					.child(
						ui::TableCell::new()
							.child(ui::format_option_percent(c.precision.production)),
					)
					.child(
						ui::TableCell::new().child(ui::format_option_percent(c.recall.production)),
					)
			})));
		ui::S2::new().child(title).child(p).child(table).into_node()
	}
}
