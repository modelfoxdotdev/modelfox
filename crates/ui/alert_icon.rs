use super::alert::Level;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct AlertIcon {
	pub alert: String,
	pub level: Level,
	#[children]
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
		let alert_message_class = classes!("alert-icon-message", level_class);
		let alert_icon_class = classes!("alert-icon", level_class);
		div()
			.class("alert-icon-container")
			.child(div().class(alert_message_class).child(self.alert))
			.child(div().class(alert_icon_class).child(self.children))
			.into_node()
	}
}
