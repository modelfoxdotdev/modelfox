use pinwheel::prelude::*;
use tangram_app_core::alerts::AlertData;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::page_heading::PageHeading;
use tangram_ui as ui;

pub struct Page {
	pub alert: AlertData,
	pub alert_id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let alert_data_table = AlertDataTable::from(self.alert);
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new()
								.child(ui::H1::new().child(format!("Alert {}", self.alert_id))),
						)
						.child(alert_data_table),
				),
			)
			.into_node()
	}
}

struct AlertDataTable {
	id: String,
	datetime: String,
	cadence: String,
	metric: String,
	mode: String,
	production_value: String,
	training_value: String,
	difference: String,
	methods: String,
}

impl Component for AlertDataTable {
	fn into_node(self) -> Node {
		let row = |name: &str, value: String| {
			ui::TableRow::new()
				.child(ui::TableCell::new().child(name.to_owned()))
				.child(ui::TableCell::new().child(value))
		};
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Attribute"))
						.child(ui::TableHeaderCell::new().child("Value")),
				),
			)
			.child(
				ui::TableBody::new()
					.child(row("ID", self.id))
					.child(row("Date", self.datetime))
					.child(row("Cadence", self.cadence))
					.child(row("Metric", self.metric))
					.child(row("Threshold Mode", self.mode))
					.child(row("Training Value", self.training_value))
					.child(row("Production Value", self.production_value))
					.child(row("Difference", self.difference))
					.child(row("Alert Methods", self.methods)),
			)
			.into_node()
	}
}

impl From<AlertData> for AlertDataTable {
	fn from(alert_data: AlertData) -> Self {
		let methods = alert_data.monitor.methods;
		let methods = match methods.len() {
			0 => "N/A".to_owned(),
			1 => methods[0].to_string(),
			_ => methods
				.iter()
				.map(|method| method.to_string())
				.collect::<Vec<String>>()
				.join(", "),
		};
		Self {
			id: alert_data.id.to_string(),
			datetime: time::OffsetDateTime::from_unix_timestamp(alert_data.timestamp)
				.expect("Unreadable timestamp")
				.to_string(),
			cadence: alert_data.monitor.cadence.to_string(),
			metric: alert_data.result.metric.to_string(),
			mode: alert_data.monitor.threshold.mode.to_string(),
			production_value: alert_data.result.production_value.to_string(),
			training_value: alert_data.result.training_value.to_string(),
			difference: alert_data.result.difference.to_string(),
			methods,
		}
	}
}
