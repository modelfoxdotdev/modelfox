use pinwheel::prelude::*;

pub fn main() {
	tangram_ui::client_start();
	hydrate::<tangram_charts::components::LineChart>("loss");
	hydrate::<tangram_charts::components::BarChart>("feature_importances");
}
