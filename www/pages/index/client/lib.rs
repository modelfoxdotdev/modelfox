use pinwheel::prelude::*;
use tangram_charts::{
	bar_chart::BarChart, box_chart::BoxChart, components::hydrate_chart,
	feature_contributions_chart::FeatureContributionsChart, line_chart::LineChart,
};
use tangram_ui as ui;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	console_log::init_with_level(log::Level::Info).unwrap();
	ui::boot_code_select();
	hydrate_chart::<LineChart>("pr-curve");
	hydrate_chart::<LineChart>("roc-curve");
	hydrate_chart::<FeatureContributionsChart>("production-explanations");
	hydrate_chart::<BarChart>("production-stats-enum");
	hydrate_chart::<BoxChart>("production-stats-number");
	hydrate_chart::<LineChart>("production-accuracy");
	hydrate_chart::<LineChart>("production-precision");
	hydrate::<tangram_www_index_common::tuning::Tuning>("tuning");
}
