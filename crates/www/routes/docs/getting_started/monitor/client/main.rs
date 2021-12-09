use pinwheel::prelude::*;
use tangram_ui as ui;
use web_sys as dom;

pub fn main() {
	tangram_ui::client_start();
	let document = dom::window().unwrap().document().unwrap();
	ui::boot_code_select();
	if document.get_element_by_id("enum_overall").is_some() {
		hydrate::<tangram_charts::components::BarChart>("enum_overall");
	}
	if document
		.get_element_by_id("production-explanations")
		.is_some()
	{
		hydrate::<tangram_charts::components::FeatureContributionsChart>("production-explanations");
	}
}
