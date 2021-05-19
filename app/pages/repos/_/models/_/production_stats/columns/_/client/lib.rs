use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, components::hydrate_chart};
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let document = dom::window().unwrap().document().unwrap();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	if document.get_element_by_id("number_intervals").is_some() {
		hydrate_chart::<BoxChart>("number_intervals");
	}
	if document.get_element_by_id("number_overall").is_some() {
		hydrate_chart::<BoxChart>("number_overall");
	}
	if document.get_element_by_id("enum_overall").is_some() {
		hydrate_chart::<BarChart>("enum_overall");
	}
	if document.get_element_by_id("text_overall").is_some() {
		hydrate_chart::<BarChart>("text_overall");
	}
}
