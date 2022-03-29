use pinwheel::prelude::*;
use modelfox_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use modelfox_app_ui::page_heading::PageHeading;
use modelfox_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub alerts_table: Option<AlertsTable>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let alerts_table_or_empty_message = if let Some(alerts_table) = self.alerts_table {
			alerts_table.into_node()
		} else {
			ui::Card::new()
				.child(ui::P::new().child("No recorded alerts for this model."))
				.into_node()
		};
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new("Alerts History".to_string())))
						.child(alerts_table_or_empty_message),
				),
			)
			.into_node()
	}
}

pub struct AlertsTable {
	pub rows: Vec<AlertsTableRow>,
}

pub struct AlertsTableRow {
	pub alert_type: String,
	pub href: String,
	pub range: String,
	pub last_updated: String,
}

impl Component for AlertsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Type"))
						.child(ui::TableHeaderCell::new().child("Monitor Range"))
						.child(ui::TableHeaderCell::new().child("Recorded")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(
							ui::TableCell::new().child(
								ui::Link::new()
									.href(format!("./{}", row.href))
									.child(row.alert_type),
							),
						)
						.child(ui::TableCell::new().child(row.range))
						.child(ui::TableCell::new().child(row.last_updated))
				})),
			)
			.into_node()
	}
}
