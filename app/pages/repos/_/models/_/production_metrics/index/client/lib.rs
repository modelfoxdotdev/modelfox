use tangram_charts::{components::hydrate_chart, line_chart::LineChart};
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	let window = window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("mse").is_some() {
		hydrate_chart::<LineChart>("mse");
	}
	if document.get_element_by_id("accuracy").is_some() {
		hydrate_chart::<LineChart>("accuracy");
	}
}
