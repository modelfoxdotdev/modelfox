use modelfox_ui as ui;
use pinwheel::prelude::*;
use web_sys as dom;

pub fn main() {
	modelfox_ui::client_start();
	let document = dom::window().unwrap().document().unwrap();
	ui::boot_code_select();
	if document.get_element_by_id("enum_overall").is_some() {
		hydrate::<modelfox_charts::components::BarChart>("enum_overall");
	}
	if document
		.get_element_by_id("production-explanations")
		.is_some()
	{
		hydrate::<modelfox_charts::components::FeatureContributionsChart>(
			"production-explanations",
		);
	}
}
