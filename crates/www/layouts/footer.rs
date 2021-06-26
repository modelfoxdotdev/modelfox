use pinwheel::prelude::*;
use tangram_www_ui::logo::{Logo, LogoColorScheme};

pub struct Footer;

impl Component for Footer {
	fn into_node(self) -> Node {
		div()
			.class("footer-wrapper")
			.child(
				Logo::new()
					.class("footer-logo".to_owned())
					.color_scheme(LogoColorScheme::Multi),
			)
			.child(p().class("footer-copyright").child("Tangram © 2020"))
			.into_node()
	}
}
