use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Card {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Card {
	fn into_node(self) -> Node {
		div().class("card").child(self.children).into_node()
	}
}
