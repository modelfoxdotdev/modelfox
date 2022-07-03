use modelfox_app_layouts::{auth_layout::AuthLayout, document::Document};
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub stage: Option<Stage>,
	pub email: Option<String>,
	pub error: Option<String>,
}

#[derive(PartialEq, Eq)]
pub enum Stage {
	Email,
	Code,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let stage = self.stage.unwrap_or(Stage::Email);
		Document::new()
			.child(
				AuthLayout::new().child(
					ui::Form::new()
						.post(true)
						.child(
							self.error
								.map(|error| ui::Alert::new(ui::Level::Danger).child(error)),
						)
						.child(
							ui::TextField::new()
								.autocomplete("username".to_owned())
								.name("email".to_owned())
								.placeholder("Email".to_owned())
								.value(self.email),
						)
						.child(if stage == Stage::Code {
							Some(
								ui::TextField::new()
									.name("code".to_owned())
									.placeholder("Code".to_owned()),
							)
						} else {
							None
						})
						.child(
							ui::Button::new()
								.button_type(ui::ButtonType::Submit)
								.child("Login"),
						)
						.child(if stage == Stage::Code {
							Some(
								div()
									.class("login-code-message")
									.child(
										"We emailed you a code. Copy and paste it above to log in.",
									)
									.into_node(),
							)
						} else {
							None
						}),
				),
			)
			.into_node()
	}
}
