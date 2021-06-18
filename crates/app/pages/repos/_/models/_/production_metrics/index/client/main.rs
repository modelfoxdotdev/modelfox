use pinwheel::prelude::*;
use tangram_ui as ui;
use web_sys as dom;

pub fn main() {
	tangram_client::client_start();
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
