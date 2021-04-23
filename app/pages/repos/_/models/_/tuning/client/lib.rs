use num::ToPrimitive;
use tangram_app_tuning_common::ClientProps;
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*, JsCast};
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = window().unwrap();
	let document = window.document().unwrap();
	let tuning_page = document
		.get_element_by_id("tuning_page")
		.map(|element| element.dyn_into::<HtmlElement>().unwrap());

	if let Some(tuning_page) = tuning_page {
		boot_tuning_page(tuning_page)
	}
}

fn boot_tuning_page(tuning_page: HtmlElement) {
	let window = window().unwrap();
	let document = window.document().unwrap();
	let client_props = tuning_page.dataset().get("props").unwrap();
	let client_props: ClientProps = serde_json::from_str(&client_props).unwrap();
	let thresholds = client_props
		.threshold_metrics
		.iter()
		.map(|metric| metric.threshold)
		.collect::<Vec<f32>>();
	let slider_formatter_thresholds = thresholds.clone();
	let slider_formatter: Box<dyn Fn(usize) -> String> =
		Box::new(move |value: usize| format!("{:.3}", slider_formatter_thresholds[value]));
	ui::boot_slider("tuning_slider", Some(slider_formatter));
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		if let Some(current_target) = event.current_target() {
			let current_target = &current_target.dyn_into::<HtmlInputElement>().unwrap();
			let value: usize = current_target.value().parse().unwrap();
			let threshold_metrics = &client_props.threshold_metrics[value];
			let baseline_metrics = &client_props.baseline_metrics;
			let threshold = thresholds[value];
			ui::update_number("tuning-threshold", threshold.to_string());
			ui::update_number_comparison_chart(
				"tuning-accuracy",
				Some(baseline_metrics.accuracy.to_f32().unwrap()),
				Some(threshold_metrics.accuracy.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-precision",
				baseline_metrics
					.precision
					.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.precision
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-recall",
				baseline_metrics.recall.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.recall
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_number_comparison_chart(
				"tuning-f1-score",
				baseline_metrics
					.f1_score
					.map(|value| value.to_f32().unwrap()),
				threshold_metrics
					.f1_score
					.map(|value| value.to_f32().unwrap()),
			);
			ui::update_confusion_matrix_comparison(
				"tuning-confusion-matrix-comparison",
				ui::ConfusionMatrixComparisonValue {
					true_positive: baseline_metrics.true_positives_fraction,
					false_positive: baseline_metrics.false_positives_fraction,
					true_negative: baseline_metrics.true_negatives_fraction,
					false_negative: baseline_metrics.false_negatives_fraction,
				},
				ui::ConfusionMatrixComparisonValue {
					true_positive: threshold_metrics.true_positives_fraction,
					false_positive: threshold_metrics.false_positives_fraction,
					true_negative: threshold_metrics.true_negatives_fraction,
					false_negative: threshold_metrics.false_negatives_fraction,
				},
			);
		}
	}));
	let slider = document
		.get_element_by_id("tuning_slider")
		.unwrap()
		.dyn_into::<HtmlInputElement>()
		.unwrap();
	slider
		.add_event_listener_with_callback("input", callback_fn.as_ref().unchecked_ref())
		.unwrap();
	callback_fn.forget();
}
