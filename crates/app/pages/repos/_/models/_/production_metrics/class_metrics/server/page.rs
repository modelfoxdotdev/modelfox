use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::{
	class_select_field::ClassSelectField,
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

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub inner: Inner,
}

pub struct Inner {
	pub class_metrics: Vec<ClassMetricsEntry>,
	pub class: String,
	pub classes: Vec<String>,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub id: String,
	pub overall: OverallClassMetrics,
}

#[derive(Clone)]
pub struct ClassMetricsEntry {
	pub class_name: String,
	pub intervals: Vec<IntervalEntry>,
}

#[derive(Clone)]
pub struct IntervalEntry {
	pub label: String,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

#[derive(Clone)]
pub struct OverallClassMetrics {
	pub class_metrics: Vec<OverallClassMetricsEntry>,
	pub label: String,
}

#[derive(Clone)]
pub struct OverallClassMetricsEntry {
	pub class_name: String,
	pub confusion_matrix_training_production_comparison:
		ConfusionMatrixTrainingProductionComparison,
	pub confusion_matrix: ConfusionMatrix,
	pub training: Metrics,
	pub production: Option<Metrics>,
}

#[derive(Clone)]
pub struct Metrics {
	pub f1_score: f32,
	pub precision: f32,
	pub recall: f32,
}

#[derive(Clone)]
pub struct ConfusionMatrixTrainingProductionComparison {
	pub training: ConfusionMatrixFraction,
	pub production: Option<ConfusionMatrixFraction>,
}

#[derive(Clone)]
pub struct ConfusionMatrixFraction {
	pub false_negative_fraction: f32,
	pub true_negative_fraction: f32,
	pub true_positive_fraction: f32,
	pub false_positive_fraction: f32,
}

#[derive(Clone)]
pub struct ConfusionMatrix {
	pub false_negatives: Option<usize>,
	pub true_negatives: Option<usize>,
	pub true_positives: Option<usize>,
	pub false_positives: Option<usize>,
}

#[derive(Clone)]
pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_app_production_class_metrics_client")
			.child(ModelLayout::new(self.model_layout_info).child(self.inner))
			.into_node()
	}
}

impl Component for Inner {
	fn into_node(self) -> Node {
		let selected_class_index = self
			.classes
			.iter()
			.position(|class| class == &self.class)
			.unwrap();
		let selected_class_interval_metrics = self.class_metrics[selected_class_index].clone();
		let selected_class_overall_metrics =
			self.overall.class_metrics[selected_class_index].clone();
		let intervals = selected_class_interval_metrics.intervals;
		let overall_training_metrics = selected_class_overall_metrics.training;
		let overall_production_metrics = selected_class_overall_metrics.production;
		ui::S1::new()
			.child(ui::H1::new().child("Production Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("".to_owned(), false).child("Overview"))
					.child(
						ui::TabLink::new("class_metrics".to_owned(), true).child("Class Metrics"),
					),
			)
			.child(
				ui::Form::new()
					.child(DateWindowSelectField::new(self.date_window))
					.child(ClassSelectField {
						class: self.class.clone(),
						classes: self.classes.clone(),
					})
					.child(
						noscript().child(
							ui::Button::new()
								.button_type(ui::ButtonType::Submit)
								.child("Submit"),
						),
					),
			)
			.child(PrecisionRecallSection {
				date_window_interval: self.date_window_interval,
				intervals,
				overall_training_metrics,
				overall_production_metrics,
			})
			.child(ConfusionMatrixSection {
				class: self.class.to_owned(),
				confusion_matrix: selected_class_overall_metrics.confusion_matrix,
			})
			.child(ProductionTrainingSection {
				class: self.class,
				confusion_matrix_training_production_comparison: selected_class_overall_metrics
					.confusion_matrix_training_production_comparison,
			})
			.into_node()
	}
}

struct PrecisionRecallSection {
	date_window_interval: DateWindowInterval,
	intervals: Vec<IntervalEntry>,
	overall_training_metrics: Metrics,
	overall_production_metrics: Option<Metrics>,
}

impl Component for PrecisionRecallSection {
	fn into_node(self) -> Node {
		let precision_interval_chart_title =
			interval_chart_title(&self.date_window_interval, "Precision".to_owned());
		let recall_interval_chart_title =
			interval_chart_title(&self.date_window_interval, "Recall".to_owned());
		let f1_score_interval_chart_title =
			interval_chart_title(&self.date_window_interval, "F1 Score".to_owned());
		let chart_labels = self
			.intervals
			.iter()
			.map(|interval| interval.label.clone())
			.collect::<Vec<_>>();

		let precision_chart_series = vec![
			LineChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: Finite::new(interval.precision.training.to_f64().unwrap()).ok(),
					})
					.collect::<Vec<_>>(),
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Training Precision".to_owned()),
			},
			LineChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: interval
							.precision
							.production
							.and_then(|precision| Finite::new(precision.to_f64().unwrap()).ok()),
					})
					.collect::<Vec<_>>(),
				line_style: None,
				point_style: None,
				title: Some("Production Precision".to_owned()),
			},
		];
		let recall_chart_series = vec![
			LineChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: Finite::new(interval.recall.training.to_f64().unwrap()).ok(),
					})
					.collect::<Vec<_>>(),
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Training Recall".to_owned()),
			},
			LineChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: interval
							.recall
							.production
							.and_then(|recall| Finite::new(recall.to_f64().unwrap()).ok()),
					})
					.collect::<Vec<_>>(),
				line_style: None,
				point_style: None,
				title: Some("Production Recall".to_owned()),
			},
		];
		let f1_score_chart_series = vec![
			LineChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: Finite::new(interval.f1_score.training.to_f64().unwrap()).ok(),
					})
					.collect::<Vec<_>>(),
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Training F1 Score".to_owned()),
			},
			LineChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.intervals
					.iter()
					.enumerate()
					.map(|(index, interval)| LineChartPoint {
						x: Finite::new(index.to_f64().unwrap()).unwrap(),
						y: interval
							.f1_score
							.production
							.and_then(|f1_score| Finite::new(f1_score.to_f64().unwrap()).ok()),
					})
					.collect::<Vec<_>>(),
				line_style: None,
				point_style: None,
				title: Some("Production F1 Score".to_owned()),
			},
		];
		ui::S2::new()
			.child(ui::H2::new().child("Precision and Recall"))
			.child(
				MetricsRow::new()
					.child(
						ui::NumberComparisonCard::new(
							Some(self.overall_training_metrics.precision),
							self.overall_production_metrics
								.as_ref()
								.map(|value| value.precision),
						)
						.color_a(TRAINING_COLOR.to_owned())
						.color_b(PRODUCTION_COLOR.to_owned())
						.title("Precision".to_owned())
						.value_a_title("Training".to_owned())
						.value_b_title("Production".to_owned())
						.number_formatter(ui::NumberFormatter::Percent(Default::default())),
					)
					.child(
						ui::NumberComparisonCard::new(
							Some(self.overall_training_metrics.recall),
							self.overall_production_metrics
								.as_ref()
								.map(|value| value.recall),
						)
						.color_a(TRAINING_COLOR.to_owned())
						.color_b(PRODUCTION_COLOR.to_owned())
						.title("Recall".to_owned())
						.value_a_title("Training".to_owned())
						.value_b_title("Production".to_owned())
						.number_formatter(ui::NumberFormatter::Percent(Default::default())),
					),
			)
			.child(
				ui::Card::new().child(Dehydrate::new(
					"precision_intervals",
					LineChart::new()
						.labels(chart_labels.clone())
						.series(precision_chart_series)
						.title(precision_interval_chart_title)
						.x_axis_grid_line_interval(GridLineInterval { k: 1.0, p: 0.0 })
						.y_max(Finite::new(1.0).unwrap())
						.y_min(Finite::new(0.0).unwrap())
				)),
			)
			.child(
				ui::Card::new().child(Dehydrate::new(
					"recall_intervals",
					LineChart::new()
						.x_axis_grid_line_interval(GridLineInterval { k: 1.0, p: 0.0 })
						.y_max(Finite::new(1.0).unwrap())
						.y_min(Finite::new(0.0).unwrap())
						.labels(chart_labels.clone())
						.series(recall_chart_series)
						.title(recall_interval_chart_title)
				)),
			)
			.child(
				MetricsRow::new().child(
					ui::NumberComparisonCard::new(
						Some(self.overall_training_metrics.f1_score),
						self.overall_production_metrics
							.as_ref()
							.map(|value| value.f1_score),
					)
					.color_a(TRAINING_COLOR.to_owned())
					.color_b(PRODUCTION_COLOR.to_owned())
					.title("F1 Score".to_owned())
					.value_a_title("Training".to_owned())
					.value_b_title("Production".to_owned())
					.number_formatter(ui::NumberFormatter::Float(Default::default())),
				),
			)
			.child(
				ui::Card::new().child(Dehydrate::new(
					"f1_intervals",
					LineChart::new()
						.x_axis_grid_line_interval(GridLineInterval { k: 1.0, p: 0.0 })
						.y_axis_grid_line_interval(None)
						.labels(chart_labels)
						.series(f1_score_chart_series)
						.title(f1_score_interval_chart_title)
						.y_min(Finite::new(0.0).unwrap())
						.y_max(Finite::new(1.0).unwrap())
				)),
			)
			.into_node()
	}
}

struct ConfusionMatrixSection {
	class: String,
	confusion_matrix: ConfusionMatrix,
}

impl Component for ConfusionMatrixSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Confusion Matrix"))
			.child(ui::ConfusionMatrix {
				class_label: self.class,
				false_negatives: self.confusion_matrix.false_negatives,
				false_positives: self.confusion_matrix.false_positives,
				true_negatives: self.confusion_matrix.true_negatives,
				true_positives: self.confusion_matrix.true_positives,
			})
			.into_node()
	}
}

struct ProductionTrainingSection {
	class: String,
	confusion_matrix_training_production_comparison: ConfusionMatrixTrainingProductionComparison,
}

impl Component for ProductionTrainingSection {
	fn into_node(self) -> Node {
		let value_a = Some(ui::ConfusionMatrixComparisonValue {
			false_negative: self
				.confusion_matrix_training_production_comparison
				.training
				.false_negative_fraction,
			false_positive: self
				.confusion_matrix_training_production_comparison
				.training
				.false_positive_fraction,
			true_negative: self
				.confusion_matrix_training_production_comparison
				.training
				.true_negative_fraction,
			true_positive: self
				.confusion_matrix_training_production_comparison
				.training
				.true_positive_fraction,
		});
		let value_b = self
			.confusion_matrix_training_production_comparison
			.production
			.as_ref()
			.map(|production| ui::ConfusionMatrixComparisonValue {
				false_negative: production.false_negative_fraction,
				false_positive: production.false_positive_fraction,
				true_negative: production.true_negative_fraction,
				true_positive: production.true_positive_fraction,
			});
		ui::S2::new()
			.child(ui::H2::new().child("Production v. Training Confusion Matrix"))
			.child(ui::ConfusionMatrixComparison {
				class_label: self.class,
				color_a: TRAINING_COLOR.to_owned(),
				color_b: PRODUCTION_COLOR.to_owned(),
				value_a,
				value_a_title: "Training".to_owned(),
				value_b,
				value_b_title: "Production".to_owned(),
			})
			.into_node()
	}
}
