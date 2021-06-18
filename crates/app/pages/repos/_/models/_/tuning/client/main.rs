use pinwheel::prelude::*;

pub fn main() {
	tangram_client::client_start();
	hydrate::<tangram_app_tuning_common::Tuning>("tuning");
}
