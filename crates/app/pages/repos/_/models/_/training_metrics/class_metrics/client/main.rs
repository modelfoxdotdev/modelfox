use tangram_ui as ui;

pub fn main() {
	tangram_client::client_start();
	ui::select_field_submit_on_change("class_select_field".to_owned());
}
