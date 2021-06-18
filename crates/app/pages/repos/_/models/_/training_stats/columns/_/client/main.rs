use pinwheel::prelude::*;
use web_sys as dom;

pub fn main() {
	tangram_client::client_start();
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
