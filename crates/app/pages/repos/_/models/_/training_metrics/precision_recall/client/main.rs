use pinwheel::prelude::*;

pub fn main() {
	tangram_ui::client_start();
	hydrate::<tangram_charts::components::LineChart>("parametric_pr");
	hydrate::<tangram_charts::components::LineChart>("non_parametric_pr");
}
