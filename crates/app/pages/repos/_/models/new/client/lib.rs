use tangram_ui as ui;
use wasm_bindgen::{self, prelude::*};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
	console_error_panic_hook::set_once();
	ui::boot_file_fields();
	Ok(())
}
