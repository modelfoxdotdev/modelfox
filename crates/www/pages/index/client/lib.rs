use pinwheel::prelude::*;
use tangram_ui as ui;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	ui::boot_code_select();
	hydrate::<tangram_charts::components::LineChart>("pr-curve");
	hydrate::<tangram_charts::components::LineChart>("roc-curve");
	hydrate::<tangram_charts::components::FeatureContributionsChart>("production-explanations");
	hydrate::<tangram_charts::components::BarChart>("production-stats-enum");
	hydrate::<tangram_charts::components::BoxChart>("production-stats-number");
	hydrate::<tangram_charts::components::LineChart>("production-accuracy");
	hydrate::<tangram_charts::components::LineChart>("production-precision");
	hydrate::<tangram_www_index_common::tuning::Tuning>("tuning");
}
