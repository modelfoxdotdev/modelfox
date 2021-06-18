pub fn client_start() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
}
