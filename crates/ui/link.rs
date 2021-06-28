use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Link {
	#[builder]
	pub class: Option<String>,
	#[builder]
	pub href: Option<String>,
	#[builder]
	pub target: Option<String>,
	#[builder]
	pub title: Option<String>,
	pub children: Vec<Node>,
}

impl Component for Link {
	fn into_node(self) -> Node {
		a().class("link")
			.class(self.class)
			.href(self.href)
			.target(self.target)
			.title(self.title)
			.child(self.children)
			.into_node()
	}
}
