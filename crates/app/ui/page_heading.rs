use pinwheel::prelude::*;

#[derive(children, Default, new)]
#[new(default)]
pub struct PageHeading {
	pub children: Vec<Node>,
}

impl Component for PageHeading {
	fn into_node(self) -> Node {
		div().class("page-heading").child(self.children).into_node()
	}
}

#[derive(children, Default, new)]
#[new(default)]
pub struct PageHeadingButtons {
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
