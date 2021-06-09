use num::ToPrimitive;
use pinwheel::prelude::*;
use std::rc::Rc;
use tangram_app_ui::colors::{BASELINE_COLOR, SELECTED_THRESHOLD_COLOR};
use tangram_ui as ui;

#[derive(ComponentBuilder, serde::Serialize, serde::Deserialize)]
pub struct Tuning {
	pub default_threshold: f32,
	pub metrics: Vec<Metrics>,
	pub default_threshold_metrics: Metrics,
	pub class: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Metrics {
	pub accuracy: f32,
	pub f1_score: Option<f32>,
	pub false_negatives_fraction: f32,
	pub false_positives_fraction: f32,
	pub precision: Option<f32>,
	pub recall: Option<f32>,
	pub threshold: f32,
	pub true_negatives_fraction: f32,
	pub true_positives_fraction: f32,
}

impl Component for Tuning {
	fn into_node(self) -> Node {
		let baseline_metrics = Rc::new(self.default_threshold_metrics);
		let metrics = Rc::new(self.metrics);
		let default_threshold = self.default_threshold;
		let baseline_index = metrics
			.iter()
			.position(|metrics| (metrics.threshold - default_threshold).abs() < std::f32::EPSILON)
			.unwrap();
		let selected_index: Mutable<usize> = Mutable::new(baseline_index);
		let on_change: Box<dyn Fn(f32)> = {
			clone!(selected_index);
			Box::new(move |value: f32| selected_index.set(value.to_usize().unwrap()))
		};
		let tooltip_number_formatter: Box<dyn Fn(f32) -> String> = {
			clone!(metrics);
			Box::new(move |value: f32| {
				let value = value.to_usize().unwrap();
				ui::format_float(metrics[value].threshold)
			})
		};
		let tuning_metrics_grid = div()
			.class("tuning-metrics-grid")
			.child_signal({
				clone!(baseline_metrics, metrics);
				selected_index.signal().map(move |selected_index| {
					ui::NumberComparisonCard::new(
						Some(baseline_metrics.accuracy),
						metrics[selected_index].accuracy,
					)
					.color_a(Some(BASELINE_COLOR.to_owned()))
					.color_b(Some(SELECTED_THRESHOLD_COLOR.to_owned()))
					.title("Accuracy".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Selected Threshold".to_owned())
					.number_formatter(ui::NumberFormatter::Percent(Default::default()))
				})
			})
			.child_signal({
				clone!(baseline_metrics, metrics);
				selected_index.signal().map(move |selected_index| {
					ui::NumberComparisonCard::new(
						Some(baseline_metrics.f1_score.unwrap()),
						metrics[selected_index].f1_score,
					)
					.color_a(Some(BASELINE_COLOR.to_owned()))
					.color_b(Some(SELECTED_THRESHOLD_COLOR.to_owned()))
					.title("F1 Score".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Selected Threshold".to_owned())
					.number_formatter(ui::NumberFormatter::Percent(Default::default()))
				})
			})
			.child_signal({
				clone!(baseline_metrics, metrics);
				selected_index.signal().map(move |selected_index| {
					ui::NumberComparisonCard::new(
						baseline_metrics.precision,
						metrics[selected_index].precision,
					)
					.color_a(Some(BASELINE_COLOR.to_owned()))
					.color_b(Some(SELECTED_THRESHOLD_COLOR.to_owned()))
					.title("Precision".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Selected Threshold".to_owned())
					.number_formatter(ui::NumberFormatter::Percent(Default::default()))
				})
			})
			.child_signal({
				clone!(baseline_metrics, metrics);
				selected_index.signal().map(move |selected_index| {
					ui::NumberComparisonCard::new(
						baseline_metrics.recall,
						metrics[selected_index].recall,
					)
					.color_a(Some(BASELINE_COLOR.to_owned()))
					.color_b(Some(SELECTED_THRESHOLD_COLOR.to_owned()))
					.title("Recall".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Selected Threshold".to_owned())
					.number_formatter(ui::NumberFormatter::Percent(Default::default()))
				})
			});
		let confusion_comparison_matrix = {
			let class = self.class;
			clone!(baseline_metrics, metrics);
			selected_index.signal().map(move |selected_index| {
				ui::ConfusionMatrixComparison::new(
					class.to_owned(),
					BASELINE_COLOR.to_owned(),
					SELECTED_THRESHOLD_COLOR.to_owned(),
					Some(ui::ConfusionMatrixComparisonValue {
						false_negative: baseline_metrics.false_negatives_fraction,
						false_positive: baseline_metrics.false_positives_fraction,
						true_negative: baseline_metrics.true_negatives_fraction,
						true_positive: baseline_metrics.true_positives_fraction,
					}),
					"Baseline".to_owned(),
					Some(ui::ConfusionMatrixComparisonValue {
						false_negative: metrics[selected_index].false_negatives_fraction,
						false_positive: metrics[selected_index].false_positives_fraction,
						true_negative: metrics[selected_index].true_negatives_fraction,
						true_positive: metrics[selected_index].true_positives_fraction,
					}),
					"Selected Threshold".to_owned(),
				)
			})
		};
		ui::S1::new()
			.child(ui::H1::new().child("Tuning"))
			.child(ui::P::new().child("Drag the silder to choose a threshold."))
			.child(
				ui::Slider::new(
					0.0,
					(metrics.len() - 1).to_f32().unwrap(),
					1.0,
					Box::new(selected_index.signal().map(|i| i.to_f32().unwrap())) as BoxSignal<_>,
				)
				.tooltip_number_formatter(tooltip_number_formatter)
				.on_change(on_change),
			)
			.child_signal({
				clone!(metrics);
				selected_index.signal().map(move |selected_index| {
					ui::NumberCard::new(
						"Selected Threshold".to_owned(),
						ui::format_float(metrics[selected_index].threshold),
					)
				})
			})
			.child(tuning_metrics_grid)
			.child_signal(confusion_comparison_matrix)
			.into_node()
	}
}
