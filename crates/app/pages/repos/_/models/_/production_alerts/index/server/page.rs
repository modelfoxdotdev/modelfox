use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::page_heading::{PageHeading, PageHeadingButtons};
use tangram_ui as ui;

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
				.child(ui::P::new().child("No configured alerts for this model."))
				.into_node()
		};
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new()
								.child(ui::H1::new().child("Production Alerts".to_string()))
								.child(
									PageHeadingButtons::new().child(
										ui::Button::new()
											.href("new".to_owned())
											.child("Create New Alert"),
									),
								),
						)
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
	pub id: String,
	pub last_updated: String,
}

impl Component for AlertsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Id"))
						.child(ui::TableHeaderCell::new().child("Uploaded")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(
							ui::TableCell::new().child(
								ui::Link::new()
									.href(format!("./alerts/{}/", row.id))
									.child(row.id.clone()),
							),
						)
						.child(ui::TableCell::new().child(row.last_updated))
				})),
			)
			.into_node()
	}
}
