use pinwheel::prelude::*;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	if document.get_element_by_id("probabilities").is_some() {
		hydrate::<tangram_charts::components::BarChart>("probabilities");
	}
	if document
		.get_element_by_id("regression_feature_contributions")
		.is_some()
	{
		hydrate::<tangram_charts::components::FeatureContributionsChart>(
			"regression_feature_contributions",
		);
	}
	if document
		.get_element_by_id("binary_classification_feature_contributions")
		.is_some()
	{
		hydrate::<tangram_charts::components::FeatureContributionsChart>(
			"binary_classification_feature_contributions",
		);
	}
	if document
		.get_element_by_id("multiclass_classification_feature_contributions")
		.is_some()
	{
		hydrate::<tangram_charts::components::FeatureContributionsChart>(
			"multiclass_classification_feature_contributions",
		);
	}
}
