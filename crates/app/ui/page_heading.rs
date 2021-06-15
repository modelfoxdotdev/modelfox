use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct PageHeading {
	#[children]
	pub children: Vec<Node>,
}

impl Component for PageHeading {
	fn into_node(self) -> Node {
		div().class("page-heading").child(self.children).into_node()
	}
}
#[derive(ComponentBuilder)]
pub struct PageHeadingButtons {
	#[children]
	pub children: Vec<Node>,
}

impl Component for PageHeadingButtons {
	fn into_node(self) -> Node {
		div()
			.class("page-heading-buttons")
			.child(self.children)
			.into_node()
	}
}
