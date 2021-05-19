use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, components::hydrate_chart};
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let document = dom::window().unwrap().document().unwrap();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	if document.get_element_by_id("class_select_field").is_some() {
		ui::select_field_submit_on_change("class_select_field".to_owned());
	}
	if document.get_element_by_id("prediction_count").is_some() {
		hydrate_chart::<BarChart>("prediction_count");
	}
	if document.get_element_by_id("quantiles_overall").is_some() {
		hydrate_chart::<BoxChart>("quantiles_overall");
	}
	if document.get_element_by_id("quantiles_intervals").is_some() {
		hydrate_chart::<BoxChart>("quantiles_intervals");
	}
	if document.get_element_by_id("histogram_overall").is_some() {
		hydrate_chart::<BarChart>("histogram_overall");
	}
	if document.get_element_by_id("histogram_intervals").is_some() {
		hydrate_chart::<BarChart>("histogram_intervals");
	}
}
