use pinwheel::prelude::*;
use modelfox_ui as ui;

pub fn main() {
	console_error_panic_hook::set_once();
	ui::boot_code_select();
	hydrate::<modelfox_www_docs_inspect_common::Tuning>("tuning");
}
