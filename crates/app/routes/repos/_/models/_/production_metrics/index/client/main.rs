use modelfox_ui as ui;
use pinwheel::prelude::*;
use web_sys as dom;

pub fn main() {
	modelfox_ui::client_start();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("mse").is_some() {
		hydrate::<modelfox_charts::components::LineChart>("mse");
	}
	if document.get_element_by_id("accuracy").is_some() {
		hydrate::<modelfox_charts::components::LineChart>("accuracy");
	}
}
