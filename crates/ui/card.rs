use pinwheel::prelude::*;

#[derive(children, Default, new)]
#[new(default)]
pub struct Card {
	pub children: Vec<Node>,
}

impl Component for Card {
	fn into_node(self) -> Node {
		div().class("card").child(self.children).into_node()
	}
}
