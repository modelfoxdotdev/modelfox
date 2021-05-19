use pinwheel::prelude::*;
use tangram_www_ui::logo::{Logo, LogoScheme};

#[derive(ComponentBuilder)]
pub struct Footer {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Footer {
	fn into_node(self) -> Node {
		div()
			.class("footer-wrapper")
			.child(Logo::new(
				Some("footer-logo".to_owned()),
				None,
				LogoScheme::Multi,
			))
			.child(p().class("footer-copyright").child("Tangram © 2020"))
			.into_node()
	}
}
