use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Token {
	#[builder]
	pub color: Option<String>,
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
