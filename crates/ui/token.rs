use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Token {
	#[optional]
	pub color: Option<String>,
	#[children]
	children: Vec<Node>,
}

impl Component for Token {
	fn into_node(self) -> Node {
		span()
			.class("token")
			.style(style::BACKGROUND_COLOR, self.color)
			.child(self.children)
			.into_node()
	}
}
