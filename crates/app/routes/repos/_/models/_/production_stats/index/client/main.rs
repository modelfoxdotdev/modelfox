use pinwheel::prelude::*;
use tangram_ui as ui;
use web_sys as dom;

pub fn main() {
	tangram_ui::client_start();
	let document = dom::window().unwrap().document().unwrap();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	if document.get_element_by_id("class_select_field").is_some() {
		ui::select_field_submit_on_change("class_select_field".to_owned());
	}
	if document.get_element_by_id("prediction_count").is_some() {
		hydrate::<tangram_charts::components::BarChart>("prediction_count");
	}
	if document.get_element_by_id("quantiles_overall").is_some() {
		hydrate::<tangram_charts::components::BoxChart>("quantiles_overall");
	}
	if document.get_element_by_id("quantiles_intervals").is_some() {
		hydrate::<tangram_charts::components::BoxChart>("quantiles_intervals");
	}
	if document.get_element_by_id("histogram_overall").is_some() {
		hydrate::<tangram_charts::components::BarChart>("histogram_overall");
	}
	if document.get_element_by_id("histogram_intervals").is_some() {
		hydrate::<tangram_charts::components::BarChart>("histogram_intervals");
	}
}
