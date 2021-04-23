use tangram_charts::{components::hydrate_chart, line_chart::LineChart};
use wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate_chart::<LineChart>("parametric_pr");
	hydrate_chart::<LineChart>("non_parametric_pr");
}
