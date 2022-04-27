use modelfox_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use modelfox_app_ui::page_heading::{PageHeading, PageHeadingButtons};
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub models_table: Option<ModelsTable>,
	pub title: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let models_table_or_empty_message = if let Some(models_table) = self.models_table {
			models_table.into_node()
		} else {
			ui::Card::new()
				.child(ui::P::new().child("This repository has no models."))
				.into_node()
		};
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(
							PageHeading::new().child(ui::H1::new(self.title)).child(
								PageHeadingButtons::new()
									.child(
										ui::Button::new()
											.color(ui::colors::GRAY.to_owned())
											.href("edit".to_owned())
											.child("Edit"),
									)
									.child(
										ui::Button::new()
											.href("models/new".to_owned())
											.child("Upload Model"),
									),
							),
						)
						.child(models_table_or_empty_message),
				),
			)
			.into_node()
	}
}

pub struct ModelsTable {
	pub rows: Vec<ModelsTableRow>,
}

pub struct ModelsTableRow {
	pub id: String,
	pub created_at: String,
	pub tag: Option<String>,
}

impl Component for ModelsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Id"))
						.child(ui::TableHeaderCell::new().child("Tag"))
						.child(ui::TableHeaderCell::new().child("Uploaded")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(
							ui::TableCell::new().child(
								ui::Link::new()
									.href(format!("./models/{}/", row.id))
									.child(row.id.clone()),
							),
						)
						.child(ui::TableCell::new().child(row.tag.clone()))
						.child(ui::TableCell::new().child(row.created_at))
				})),
			)
			.into_node()
	}
}
