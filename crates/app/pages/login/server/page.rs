use pinwheel::prelude::*;
use tangram_app_layouts::{auth_layout::AuthLayout, document::Document};
use tangram_ui as ui;

pub struct Page {
	pub code: bool,
	pub email: Option<String>,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
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
						.child(if self.code {
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
						.child(if self.code {
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
