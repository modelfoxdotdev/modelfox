use modelfox_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use modelfox_app_ui::page_heading::PageHeading;
use modelfox_id::Id;
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub created_at: String,
	pub model_heading: String,
	pub model_id: Id,
	pub tag: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new(self.model_heading)))
						.child(ModelInfoTable {
							model_id: self.model_id,
							created_at: self.created_at,
						})
						.child(UpdateTagForm { tag: self.tag })
						.child(DangerZone),
				),
			)
			.into_node()
	}
}

struct ModelInfoTable {
	model_id: Id,
	created_at: String,
}

impl Component for ModelInfoTable {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(
				ui::Table::new()
					.child(
						ui::TableRow::new()
							.child(ui::TableHeaderCell::new().child("Model Id"))
							.child(ui::TableCell::new().child(self.model_id.to_string())),
					)
					.child(
						ui::TableRow::new()
							.child(ui::TableHeaderCell::new().child("Uploaded At"))
							.child(ui::TableCell::new().child(self.created_at)),
					),
			)
			.into_node()
	}
}

struct UpdateTagForm {
	tag: Option<String>,
}

impl Component for UpdateTagForm {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new("Tag"))
			.child(
				ui::Form::new()
					.post(true)
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "update_tag"),
					)
					.child(
						ui::TextField::new()
							.label("Tag".to_owned())
							.name("tag".to_owned())
							.value(self.tag),
					)
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.child("Update"),
					),
			)
			.into_node()
	}
}

struct DangerZone;

impl Component for DangerZone {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new("Danger Zone"))
			.child(
				ui::Form::new()
					.post(true)
					.onsubmit("return confirm(\"Are you sure?\")".to_owned())
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "delete"),
					)
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.color(ui::colors::RED.to_owned())
							.child("Delete"),
					),
			)
			.into_node()
	}
}
