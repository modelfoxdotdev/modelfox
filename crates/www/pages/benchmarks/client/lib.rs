use pinwheel::prelude::*;
use wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	hydrate::<tangram_www_benchmarks_common::Benchmarks>("benchmarks");
}
