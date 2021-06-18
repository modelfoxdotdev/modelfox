use pinwheel::prelude::*;

pub fn main() {
	tangram_client::client_start();
	hydrate::<tangram_charts::components::LineChart>("roc");
}
