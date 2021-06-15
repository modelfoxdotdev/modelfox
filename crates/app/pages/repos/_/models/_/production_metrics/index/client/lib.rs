use pinwheel::prelude::*;
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("mse").is_some() {
		hydrate::<tangram_charts::components::LineChart>("mse");
	}
	if document.get_element_by_id("accuracy").is_some() {
		hydrate::<tangram_charts::components::LineChart>("accuracy");
	}
}
