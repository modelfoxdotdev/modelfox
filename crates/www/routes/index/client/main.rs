use modelfox_ui as ui;
use pinwheel::prelude::*;

fn main() {
	modelfox_ui::client_start();
	ui::boot_code_select();
	hydrate::<modelfox_charts::components::LineChart>("pr-curve");
	hydrate::<modelfox_charts::components::LineChart>("roc-curve");
	hydrate::<modelfox_charts::components::FeatureContributionsChart>("production-explanations");
	hydrate::<modelfox_charts::components::BarChart>("production-stats-enum");
	hydrate::<modelfox_charts::components::BoxChart>("production-stats-number");
	hydrate::<modelfox_charts::components::LineChart>("production-accuracy");
	hydrate::<modelfox_charts::components::LineChart>("production-precision");
	hydrate::<modelfox_www_index_common::tuning::Tuning>("tuning");
}
