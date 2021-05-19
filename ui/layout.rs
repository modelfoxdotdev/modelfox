use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct S1 {
	#[children]
	pub children: Vec<Node>,
}

impl Component for S1 {
	fn into_node(self) -> Node {
		div().class("s1").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct S2 {
	#[children]
	pub children: Vec<Node>,
}

impl Component for S2 {
	fn into_node(self) -> Node {
		div().class("s2").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct SpaceBetween {
	#[children]
	pub children: Vec<Node>,
}

impl Component for SpaceBetween {
	fn into_node(self) -> Node {
		div()
			.class("space-between")
			.child(self.children)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct H1 {
	#[optional]
	pub center: Option<bool>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for H1 {
	fn into_node(self) -> Node {
		let center = if self.center.unwrap_or(false) {
			Some("center")
		} else {
			None
		};
		let class = classes!(Some("h1"), center);
		h1().class(class).child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct H2 {
	#[optional]
	pub center: Option<bool>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for H2 {
	fn into_node(self) -> Node {
		let center = if self.center.unwrap_or(false) {
			Some("center")
		} else {
			None
		};
		let class = classes!(Some("h2"), center);
		h2().class(class).child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct P {
	#[children]
	pub children: Vec<Node>,
}

impl Component for P {
	fn into_node(self) -> Node {
		p().class("p").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct UnorderedList {
	#[children]
	pub children: Vec<Node>,
}

impl Component for UnorderedList {
	fn into_node(self) -> Node {
		ul().class("unordered-list")
			.child(self.children)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct OrderedList {
	#[children]
	pub children: Vec<Node>,
}

impl Component for OrderedList {
	fn into_node(self) -> Node {
		ol().class("ordered-list").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ListItem {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ListItem {
	fn into_node(self) -> Node {
		li().child(self.children).into_node()
	}
}
