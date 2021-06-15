use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Alert {
	pub level: Level,
	#[optional]
	pub title: Option<String>,
	#[children]
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
		let class = classes!("alert-wrapper", level_class);
		let title = self
			.title
			.map(|title| div().class("alert-title").child(title).into_node());
		div()
			.class(class)
			.child(title)
			.child(self.children)
			.into_node()
	}
}
