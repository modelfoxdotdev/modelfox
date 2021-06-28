use super::alert::Level;
use pinwheel::prelude::*;

#[derive(builder, children, new)]
pub struct AlertIcon {
	pub alert: String,
	pub level: Level,
	#[new(default)]
	pub children: Vec<Node>,
}

impl Component for AlertIcon {
	fn into_node(self) -> Node {
		let level_class = match self.level {
			Level::Info => "alert-icon-level-info",
			Level::Success => "alert-icon-level-success",
			Level::Warning => "alert-icon-level-warning",
			Level::Danger => "alert-icon-level-danger",
		};
		div()
			.class("alert-icon-container")
			.child(
				div()
					.class("alert-icon-message")
					.class(level_class)
					.child(self.alert),
			)
			.child(
				div()
					.class("alert-icon")
					.class(level_class)
					.child(self.children),
			)
			.into_node()
	}
}
