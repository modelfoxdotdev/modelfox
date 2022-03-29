use pinwheel::prelude::*;

pub fn main() {
	modelfox_ui::client_start();
	hydrate::<modelfox_app_tuning_common::Tuning>("tuning");
}
