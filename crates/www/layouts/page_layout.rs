use crate::layout::Layout;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct PageLayout {
	#[children]
	pub children: Vec<Node>,
}

impl Component for PageLayout {
	fn into_node(self) -> Node {
		Layout::new()
			.child(div().class("page-layout").child(self.children))
			.into_node()
	}
}
