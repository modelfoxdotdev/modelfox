use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_ui as ui;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub error: Option<String>,
	pub owner: Option<String>,
	pub owners: Option<Vec<Owner>>,
	pub title: Option<String>,
}

pub struct Owner {
	pub value: String,
	pub title: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let owner = self.owner;
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new().child(ui::H1::new("Create New Repo")).child(
						ui::Form::new()
							.post(true)
							.child(
								self.error
									.map(|error| ui::Alert::new(ui::Level::Danger).child(error)),
							)
							.child(
								ui::TextField::new()
									.label("Title".to_owned())
									.name("title".to_owned())
									.required(true)
									.value(self.title),
							)
							.child(self.owners.map(|owners| {
								ui::SelectField::new()
									.label("Owner".to_owned())
									.name("owner".to_owned())
									.options(
										owners
											.into_iter()
											.map(|owner| ui::SelectFieldOption {
												text: owner.title,
												value: owner.value,
											})
											.collect::<Vec<_>>(),
									)
									.required(true)
									.value(owner)
							}))
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
