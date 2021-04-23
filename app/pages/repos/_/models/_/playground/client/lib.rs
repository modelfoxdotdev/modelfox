use tangram_charts::{
	bar_chart::BarChart, box_chart::BoxChart, components::hydrate_chart,
	feature_contributions_chart::FeatureContributionsChart,
};
use wasm_bindgen::{self, prelude::*, JsCast};
use web_sys::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let window = window().unwrap();
	let document = window.document().unwrap();
	let bar_charts_query = document
		.query_selector_all(".column-chart[data-chart-type='bar']")
		.unwrap();
	for index in 0..bar_charts_query.length() {
		let item = bar_charts_query
			.item(index)
			.unwrap()
			.dyn_into::<Element>()
			.unwrap();
		hydrate_chart::<BarChart>(&item.id());
	}
	let box_charts_query = document
		.query_selector_all(".column-chart[data-chart-type='box']")
		.unwrap();
	for index in 0..box_charts_query.length() {
		let item = box_charts_query
			.item(index)
			.unwrap()
			.dyn_into::<Element>()
			.unwrap();
		hydrate_chart::<BoxChart>(&item.id());
	}
	if document.get_element_by_id("probabilities").is_some() {
		hydrate_chart::<BarChart>("probabilities")
	}
	if document
		.get_element_by_id("regression_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>("regression_feature_contributions")
	}
	if document
		.get_element_by_id("binary_classification_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>("binary_classification_feature_contributions")
	}
	if document
		.get_element_by_id("multiclass_classification_feature_contributions")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>(
			"multiclass_classification_feature_contributions",
		)
	}
}
