use pinwheel::prelude::*;
use wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate::<tangram_charts::components::LineChart>("parametric_pr");
	hydrate::<tangram_charts::components::LineChart>("non_parametric_pr");
}
