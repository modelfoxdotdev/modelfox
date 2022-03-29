use pinwheel::prelude::*;
use modelfox_ui as ui;

pub fn main() {
	modelfox_ui::client_start();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	ui::select_field_submit_on_change("class_select_field".to_owned());
	hydrate::<modelfox_charts::components::LineChart>("precision_intervals");
	hydrate::<modelfox_charts::components::LineChart>("recall_intervals");
	hydrate::<modelfox_charts::components::LineChart>("f1_intervals");
}
