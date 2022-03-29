use pinwheel::prelude::*;
use modelfox_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use modelfox_ui as ui;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new().child(ui::H1::new("Edit Organization")).child(
						ui::Form::new()
							.post(true)
							.child(
								ui::TextField::new()
									.label("Organization Name".to_owned())
									.name("name".to_owned()),
							)
							.child(
								ui::Button::new()
									.button_type(ui::ButtonType::Submit)
									.child("Submit"),
							),
					),
				),
			)
			.into_node()
	}
}
