use pinwheel::prelude::*;
use tangram_ui as ui;

pub fn main() {
	tangram_client::client_start();
	ui::select_field_submit_on_change("date_window_select_field".to_owned());
	ui::select_field_submit_on_change("class_select_field".to_owned());
	hydrate::<tangram_charts::components::LineChart>("precision_intervals");
	hydrate::<tangram_charts::components::LineChart>("recall_intervals");
	hydrate::<tangram_charts::components::LineChart>("f1_intervals");
}
