use tangram_charts::{
	bar_chart::BarChart, components::hydrate_chart,
	feature_contributions_chart::FeatureContributionsChart,
};
use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};
use web_sys as dom;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	let document = dom::window().unwrap().document().unwrap();
	ui::boot_code_select();
	if document.get_element_by_id("enum_overall").is_some() {
		hydrate_chart::<BarChart>("enum_overall");
	}
	if document
		.get_element_by_id("production-explanations")
		.is_some()
	{
		hydrate_chart::<FeatureContributionsChart>("production-explanations");
	}
}
