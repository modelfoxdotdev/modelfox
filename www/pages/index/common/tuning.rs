use indoc::formatdoc;
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::borrow::Cow;
use std::rc::Rc;
use tangram_ui as ui;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tuning {
	pub threshold_metrics: Vec<ThresholdMetrics>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ThresholdMetrics {
	pub accuracy: f32,
	pub f1_score: f32,
	pub false_negatives: usize,
	pub false_positives: usize,
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
	pub true_negatives: usize,
	pub true_positives: usize,
}

impl Component for Tuning {
	fn into_node(self) -> Node {
		let Tuning {
			threshold_metrics, ..
		} = self;
		let threshold_metrics = Rc::new(threshold_metrics);
		let n_thresholds = threshold_metrics.len();
		let center = n_thresholds / 2;
		let selected_index: Mutable<usize> = Mutable::new(center);
		let tooltip_number_formatter: Box<dyn Fn(f32) -> String> = {
			clone!(threshold_metrics);
			Box::new(move |value: f32| {
				let value = value.to_usize().unwrap();
				ui::format_float(threshold_metrics[value].threshold)
			})
		};
		let on_change: Box<dyn Fn(f32)> = {
			clone!(selected_index);
			Box::new(move |value: f32| selected_index.set(value.to_usize().unwrap()))
		};
		let slider = ui::Slider::new(
			0.0,
			(threshold_metrics.len() - 1).to_f32().unwrap(),
			1.0,
			Box::new(selected_index.signal().map(|i| i.to_f32().unwrap())) as BoxSignal<_>,
		)
		.tooltip_number_formatter(tooltip_number_formatter)
		.on_change(on_change);
		let accuracy = {
			clone!(threshold_metrics);
			selected_index
				.signal()
				.map(move |i| ui::format_percent(threshold_metrics[i].accuracy))
		};
		let accuracy = div().style(style::GRID_AREA, "accuracy").child_signal(
			accuracy.map(|accuracy| ui::NumberCard::new("Accuracy".to_owned(), accuracy)),
		);
		let precision = {
			clone!(threshold_metrics);
			selected_index
				.signal()
				.map(move |i| ui::format_percent(threshold_metrics[i].precision))
		};
		let precision = div().style(style::GRID_AREA, "precision").child_signal(
			precision.map(|precision| ui::NumberCard::new("Precision".to_owned(), precision)),
		);
		let recall = {
			clone!(threshold_metrics);
			selected_index
				.signal()
				.map(move |i| ui::format_percent(threshold_metrics[i].recall))
		};
		let recall = div()
			.style(style::GRID_AREA, "recall")
			.child_signal(recall.map(|recall| ui::NumberCard::new("Recall".to_owned(), recall)));
		let code = {
			clone!(threshold_metrics);
			selected_index.signal().map(move |i| {
				formatdoc!(
					r#"
						// Update your code to use the selected threshold.
						model.predict(input, {{ threshold: {:.2} }})
					"#,
					threshold_metrics[i].threshold
				)
			})
		};
		let code = code.map(|code| ui::Code::new().code(Cow::Owned(code)).into_node());
		div()
			.class("tuning-grid")
			.child(slider)
			.child(
				div()
					.class("tuning-number-chart-grid")
					.child(accuracy)
					.child(precision)
					.child(recall),
			)
			.child_signal(code)
			.into_node()
	}
}
