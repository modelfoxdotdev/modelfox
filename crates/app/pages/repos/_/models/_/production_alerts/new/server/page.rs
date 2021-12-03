use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_app_ui::page_heading::PageHeading;
use tangram_ui as ui;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_app_new_model_production_client")
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(PageHeading::new().child(ui::H1::new().child("Upload Model")))
						.child(
							ui::Form::new()
								.enc_type("multipart/form-data".to_owned())
								.post(true)
								.child(
									self.error.map(|error| {
										ui::Alert::new(ui::Level::Danger).child(error)
									}),
								)
								.child(
									ui::FileField::new()
										.label("File".to_string())
										.name("file".to_string())
										.required(true),
								)
								.child(
									ui::Button::new()
										.button_type(ui::ButtonType::Submit)
										.child("Upload"),
								),
						),
				),
			)
			.into_node()
	}
}
