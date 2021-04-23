use tangram_charts::{components::hydrate_chart, line_chart::LineChart};
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	ui::select_field_submit_on_change("class_select_field".to_owned());
	hydrate_chart::<LineChart>("precision_intervals");
	hydrate_chart::<LineChart>("recall_intervals");
	hydrate_chart::<LineChart>("f1_intervals");
}
