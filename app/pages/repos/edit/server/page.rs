use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_app_ui::page_heading::PageHeading;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
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
						.child(PageHeading::new().child(ui::H1::new().child(self.title.clone())))
						.child(UpdateTitleForm::new(self.title))
						.child(DangerZone::new()),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
struct UpdateTitleForm {
	title: String,
}

impl Component for UpdateTitleForm {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Title"))
			.child(
				ui::Form::new()
					.post(Some(true))
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
							.value(Some(self.title)),
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
