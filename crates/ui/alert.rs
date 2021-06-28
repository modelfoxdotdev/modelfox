use pinwheel::prelude::*;

#[derive(builder, children, new)]
pub struct Alert {
	pub level: Level,
	#[builder]
	#[new(default)]
	pub title: Option<String>,
	#[new(default)]
	pub children: Vec<Node>,
}

pub enum Level {
	Info,
	Success,
	Warning,
	Danger,
}

impl Component for Alert {
	fn into_node(self) -> Node {
		let level_class = match self.level {
			Level::Info => "alert-level-info",
			Level::Success => "alert-level-success",
			Level::Warning => "alert-level-warning",
			Level::Danger => "alert-level-danger",
		};
		let title = self
			.title
			.map(|title| div().class("alert-title").child(title).into_node());
		div()
			.class("alert-wrapper")
			.class(level_class)
			.child(title)
			.child(self.children)
			.into_node()
	}
}
