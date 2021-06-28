use pinwheel::prelude::*;

pub fn main() {
	tangram_ui::client_start();
	hydrate::<tangram_app_tuning_common::Tuning>("tuning");
}
