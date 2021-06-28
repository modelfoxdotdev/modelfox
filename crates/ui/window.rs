use crate as ui;
use pinwheel::prelude::*;

pub enum WindowShade {
	Code,
	Default,
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Window {
	#[builder]
	pub padding: Option<bool>,
	pub children: Vec<Node>,
}

impl Component for Window {
	fn into_node(self) -> Node {
		let padding_class = if self.padding.unwrap_or(true) {
			Some("window-body-padding")
		} else {
			None
		};
		div()
			.class("window-wrapper")
			.child(
				div()
					.class("window-topbar")
					.child(
						div()
							.class("window-topbar-button")
							.style(style::BACKGROUND_COLOR, ui::colors::RED),
					)
					.child(
						div()
							.class("window-topbar-button")
							.style(style::BACKGROUND_COLOR, ui::colors::YELLOW),
					)
					.child(
						div()
							.class("window-topbar-button")
							.style(style::BACKGROUND_COLOR, ui::colors::GREEN),
					),
			)
			.child(
				div()
					.class("window-body")
					.class(padding_class)
					.child(self.children),
			)
			.into_node()
	}
}
