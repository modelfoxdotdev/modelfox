use pinwheel::prelude::*;
use tangram_ui as ui;

fn main() {
	tangram_client::client_start();
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
