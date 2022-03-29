use pinwheel::prelude::*;
use modelfox_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use modelfox_app_ui::page_heading::{PageHeading, PageHeadingButtons};
use modelfox_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub monitors_table: Option<MonitorsTable>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let monitors_table_or_empty_message = if let Some(monitors_table) = self.monitors_table {
			monitors_table.into_node()
		} else {
			ui::Card::new()
				.child(ui::P::new().child("No configured monitors for this model."))
				.into_node()
		};
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new()
								.child(ui::H1::new("Production Monitors".to_string()))
								.child(
									PageHeadingButtons::new().child(
										ui::Button::new()
											.href("new".to_owned())
											.child("Create New Monitor"),
									),
								),
						)
						.child(monitors_table_or_empty_message),
				),
			)
			.into_node()
	}
}

pub struct MonitorsTable {
	pub rows: Vec<MonitorsTableRow>,
}

pub struct MonitorsTableRow {
	pub id: String,
	pub name: String,
	pub last_updated: String,
}

impl Component for MonitorsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Name"))
						.child(ui::TableHeaderCell::new().child("Uploaded")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(
							ui::TableCell::new().child(
								ui::Link::new()
									.href(format!("./{}/edit", row.id))
									.child(row.name),
							),
						)
						.child(ui::TableCell::new().child(row.last_updated))
				})),
			)
			.into_node()
	}
}
