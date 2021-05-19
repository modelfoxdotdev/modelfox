use pinwheel::prelude::*;
use tangram_app_ui::logo::{Logo, LogoScheme};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct AuthLayout {
	#[children]
	pub children: Vec<Node>,
}

impl Component for AuthLayout {
	fn into_node(self) -> Node {
		div()
			.class("auth-layout")
			.child(
				div()
					.class("auth-layout-logo-wrapper")
					.child(Logo::new(LogoScheme::Multi)),
			)
			.child(
				div()
					.class("auth-layout-card-wrapper")
					.child(ui::Card::new().child(self.children)),
			)
			.into_node()
	}
}
