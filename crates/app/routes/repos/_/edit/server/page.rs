use modelfox_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use modelfox_app_ui::page_heading::PageHeading;
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub title: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new(self.title.clone())))
						.child(UpdateTitleForm { title: self.title })
						.child(DangerZone),
				),
			)
			.into_node()
	}
}

struct UpdateTitleForm {
	title: String,
}

impl Component for UpdateTitleForm {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new("Title"))
			.child(
				ui::Form::new()
					.post(true)
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "update_title"),
					)
					.child(
						ui::TextField::new()
							.label("Title".to_owned())
							.name("title".to_owned())
							.value(self.title),
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
