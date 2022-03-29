use pinwheel::prelude::*;
use wasm_bindgen::JsCast;
use web_sys as dom;

pub fn main() {
	modelfox_ui::client_start();
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	let charts_query = document
		.query_selector_all(".predict-column-chart-wrapper")
		.unwrap();
	for index in 0..charts_query.length() {
		let item = charts_query
			.item(index)
			.unwrap()
			.dyn_into::<dom::Element>()
			.unwrap()
			.parent_element()
			.unwrap();
		hydrate::<modelfox_app_playground_common::ColumnChart>(&item.id());
	}
	if document.get_element_by_id("probabilities").is_some() {
		hydrate::<modelfox_charts::components::BarChart>("probabilities")
	}
	if document
		.get_element_by_id("regression_feature_contributions")
		.is_some()
	{
		hydrate::<modelfox_charts::components::FeatureContributionsChart>(
			"regression_feature_contributions",
		)
	}
	if document
		.get_element_by_id("binary_classification_feature_contributions")
		.is_some()
	{
		hydrate::<modelfox_charts::components::FeatureContributionsChart>(
			"binary_classification_feature_contributions",
		)
	}
	if document
		.get_element_by_id("multiclass_classification_feature_contributions")
		.is_some()
	{
		hydrate::<modelfox_charts::components::FeatureContributionsChart>(
			"multiclass_classification_feature_contributions",
		)
	}
}
