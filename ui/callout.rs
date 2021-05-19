use crate::alert::Level;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Callout {
	pub level: Level,
	#[optional]
	pub title: Option<String>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for Callout {
	fn into_node(self) -> Node {
		let level_class = match self.level {
			Level::Danger => "callout-wrapper-danger",
			Level::Info => "callout-wrapper-info",
			Level::Warning => "callout-wrapper-warning",
			Level::Success => "callout-wrapper-success",
		};
		let class = classes!("callout-wrapper", level_class);
		div()
			.attribute("class", class)
			.child({
				self.title
					.map(|title| div().class("callout-title").child(title))
			})
			.child(div().class("callout-inner").child(self.children))
			.into_node()
	}
}
