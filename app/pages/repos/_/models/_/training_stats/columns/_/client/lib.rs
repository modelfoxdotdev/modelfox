use pinwheel::prelude::*;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("enum_histogram").is_some() {
		hydrate::<tangram_charts::components::BarChart>("enum_histogram");
	}
	if document.get_element_by_id("number_quantiles").is_some() {
		hydrate::<tangram_charts::components::BoxChart>("number_quantiles");
	}
	if document.get_element_by_id("number_histogram").is_some() {
		hydrate::<tangram_charts::components::BarChart>("number_histogram");
	}
	if document.get_element_by_id("ngram_histogram").is_some() {
		hydrate::<tangram_charts::components::BarChart>("ngram_histogram");
	}
}
