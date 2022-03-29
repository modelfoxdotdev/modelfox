use pinwheel::prelude::*;

pub fn main() {
	modelfox_ui::client_start();
	hydrate::<modelfox_charts::components::LineChart>("parametric_pr");
	hydrate::<modelfox_charts::components::LineChart>("non_parametric_pr");
}
