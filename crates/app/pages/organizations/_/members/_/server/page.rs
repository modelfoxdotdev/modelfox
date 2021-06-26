use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_ui as ui;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub can_delete: bool,
	pub is_admin: bool,
	pub member_email: String,
	pub remove_button_text: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child("Edit Member"))
						.child(
							ui::S2::new()
								.child(
									ui::TextField::new()
										.label("Email".to_owned())
										.disabled(Some(true))
										.value(Some(self.member_email)),
								)
								.child(
									ui::CheckboxField::new()
										.label("Admin".to_owned())
										.disabled(Some(true))
										.name("is_admin".to_owned())
										.checked(Some(self.is_admin)),
								),
						)
						.child(if self.can_delete {
							Some(DangerZone {
								remove_button_text: self.remove_button_text,
							})
						} else {
							None
						}),
				),
			)
			.into_node()
	}
}

struct DangerZone {
	remove_button_text: String,
}

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
							.child(self.remove_button_text),
					),
			)
			.into_node()
	}
}
