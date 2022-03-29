use pinwheel::prelude::*;

pub fn main() {
	modelfox_ui::client_start();
	hydrate::<modelfox_charts::components::LineChart>("loss");
	hydrate::<modelfox_charts::components::BarChart>("feature_importances");
}
