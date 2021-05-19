use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Link {
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub href: Option<String>,
	#[optional]
	pub target: Option<String>,
	#[optional]
	pub title: Option<String>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for Link {
	fn into_node(self) -> Node {
		let class = classes!("link", self.class);
		a().class(class)
			.href(self.href)
			.target(self.target)
			.title(self.title)
			.child(self.children)
			.into_node()
	}
}
