use pinwheel::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	hydrate::<tangram_app_tuning_common::Tuning>("tuning");
}
