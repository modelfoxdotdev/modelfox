use pinwheel::prelude::*;
use modelfox_app_ui::logo::Logo;
use modelfox_ui as ui;

#[derive(children, Default, new)]
#[new(default)]
pub struct AuthLayout {
	pub children: Vec<Node>,
}

impl Component for AuthLayout {
	fn into_node(self) -> Node {
		div()
			.class("auth-layout")
			.child(div().class("auth-layout-logo-wrapper").child(Logo::new()))
			.child(
				div()
					.class("auth-layout-card-wrapper")
					.child(ui::Card::new().child(self.children)),
			)
			.into_node()
	}
}
