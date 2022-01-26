use pinwheel::prelude::*;
use tangram_app_core::{alert::Alert, monitor::MonitorThresholdMode};
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::{
	colors::{BASELINE_COLOR, TRAINING_COLOR},
	page_heading::PageHeading,
};
use tangram_ui as ui;

pub struct Page {
	pub alert: Alert,
	pub alert_id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub error: Option<String>,
}

pub fn alert_description(alert: &Alert) -> String {
	let time_range = alert.formated_time_range();
	let cadence = alert.monitor.cadence;
	let metric = alert.monitor.threshold.metric;
	let production_value = alert.result.production_value;
	let training_value = alert.result.training_value;
	let difference = alert.result.difference;
	let method_str = alert
		.monitor
		.methods
		.iter()
		.map(|method| method.to_string())
		.collect::<Vec<String>>()
		.join(",");
	format!("During the period from {time_range}, this {cadence} {metric} alert observed a production value of {production_value}, which is {difference} difference from the training metric {training_value}.  Alerts were sent to the following methods: {method_str}.")
}

impl Component for Page {
	fn into_node(self) -> Node {
		let formatter = match self.alert.monitor.threshold.mode {
			MonitorThresholdMode::Absolute => ui::NumberFormatter::Float(Default::default()),
			MonitorThresholdMode::Percentage => ui::NumberFormatter::Percent(Default::default()),
		};
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new(self.alert.title())))
						.child(ui::P::new().child(alert_description(&self.alert)))
						.child(
							ui::NumberComparisonCard::new(
								Some(self.alert.training_value()),
								Some(self.alert.production_value()),
							)
							.color_a(BASELINE_COLOR.to_owned())
							.color_b(TRAINING_COLOR.to_owned())
							.title(self.alert.metric().to_string())
							.value_a_title("Training Metric".to_owned())
							.value_b_title("Production Metric".to_owned())
							.number_formatter(formatter),
						),
				),
			)
			.into_node()
	}
}
