use modelfox_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new("Create New Organization"))
						.child(
							ui::Form::new()
								.post(true)
								.child(
									ui::TextField::new()
										.label("Name".to_owned())
										.name("name".to_owned())
										.required(true),
								)
								.child(
									ui::Button::new()
										.button_type(ui::ButtonType::Submit)
										.child("Create"),
								),
						),
				),
			)
			.into_node()
	}
}
