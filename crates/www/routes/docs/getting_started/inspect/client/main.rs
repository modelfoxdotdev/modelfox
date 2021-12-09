use pinwheel::prelude::*;
use tangram_ui as ui;

pub fn main() {
	console_error_panic_hook::set_once();
	ui::boot_code_select();
	hydrate::<tangram_www_docs_inspect_common::Tuning>("tuning");
}
