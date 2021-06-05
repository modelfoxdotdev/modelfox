use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_app_ui::page_heading::PageHeading;
use tangram_id::Id;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub model_id: Id,
	pub model_heading: String,
	pub tag: Option<String>,
	pub created_at: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new().child(self.model_heading)))
						.child(ModelInfoTable::new(self.model_id, self.created_at))
						.child(UpdateTagForm::new(self.tag))
						.child(DangerZone::new()),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
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
							.child(ui::TableCell::new().child(Some(self.model_id.to_string()))),
					)
					.child(
						ui::TableRow::new()
							.child(ui::TableHeaderCell::new().child("Uploaded At"))
							.child(ui::TableCell::new().child(Some(self.created_at))),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct UpdateTagForm {
	tag: Option<String>,
}

impl Component for UpdateTagForm {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Tag"))
			.child(
				ui::Form::new()
					.post(Some(true))
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
							.button_type(Some(ui::ButtonType::Submit))
							.child("Update"),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct DangerZone;

impl Component for DangerZone {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Danger Zone"))
			.child(
				ui::Form::new()
					.post(Some(true))
					.onsubmit("return confirm(\"Are you sure?\")".to_owned())
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "delete"),
					)
					.child(
						ui::Button::new()
							.button_type(Some(ui::ButtonType::Submit))
							.color(Some(ui::colors::RED.to_owned()))
							.child("Delete"),
					),
			)
			.into_node()
	}
}
