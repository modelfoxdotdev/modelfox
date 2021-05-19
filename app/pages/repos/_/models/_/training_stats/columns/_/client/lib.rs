use tangram_charts::{bar_chart::BarChart, box_chart::BoxChart, components::hydrate_chart};
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("enum_histogram").is_some() {
		hydrate_chart::<BarChart>("enum_histogram");
	}
	if document.get_element_by_id("number_quantiles").is_some() {
		hydrate_chart::<BoxChart>("number_quantiles");
	}
	if document.get_element_by_id("number_histogram").is_some() {
		hydrate_chart::<BarChart>("number_histogram");
	}
	if document.get_element_by_id("ngram_histogram").is_some() {
		hydrate_chart::<BarChart>("ngram_histogram");
	}
}
