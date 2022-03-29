use modelfox_ui as ui;

pub fn main() {
	modelfox_ui::client_start();
	ui::select_field_submit_on_change("class_select_field".to_owned());
}
