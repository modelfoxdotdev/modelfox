use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub model_id: String,
	pub model_layout_info: ModelLayoutInfo,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(ModelLayout::new(self.model_layout_info))
			.into_node()
	}
}

pub struct AlertsTable {
	pub rows: Vec<AlertsTableRow>,
}

pub struct AlertsTableRow {
	pub id: String,
	pub created_at: String,
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
						.child(ui::TableCell::new().child(row.created_at))
				})),
			)
			.into_node()
	}
}
