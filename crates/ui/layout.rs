use pinwheel::prelude::*;
use std::borrow::Cow;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct S1 {
	pub children: Vec<Node>,
}

impl Component for S1 {
	fn into_node(self) -> Node {
		div().class("s1").child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct S2 {
	pub children: Vec<Node>,
}

impl Component for S2 {
	fn into_node(self) -> Node {
		div().class("s2").child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct SpaceBetween {
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

#[derive(builder, new)]
pub struct H1 {
	pub title: Cow<'static, str>,
	#[new(default)]
	#[builder]
	pub center: Option<bool>,
}

impl Component for H1 {
	fn into_node(self) -> Node {
		let center = if self.center.unwrap_or(false) {
			Some("center")
		} else {
			None
		};
		h1().class("h1").class(center).child(self.title).into_node()
	}
}

#[derive(builder, new)]
pub struct H2 {
	pub title: Cow<'static, str>,
	#[new(default)]
	#[builder]
	pub center: Option<bool>,
}

impl Component for H2 {
	fn into_node(self) -> Node {
		let center = if self.center.unwrap_or(false) {
			Some("center")
		} else {
			None
		};
		h2().class("h2").class(center).child(self.title).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct P {
	pub children: Vec<Node>,
}

impl Component for P {
	fn into_node(self) -> Node {
		p().class("p").child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct UnorderedList {
	pub children: Vec<Node>,
}

impl Component for UnorderedList {
	fn into_node(self) -> Node {
		ul().class("unordered-list")
			.child(self.children)
			.into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct OrderedList {
	pub children: Vec<Node>,
}

impl Component for OrderedList {
	fn into_node(self) -> Node {
		ol().class("ordered-list").child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct ListItem {
	pub children: Vec<Node>,
}

impl Component for ListItem {
	fn into_node(self) -> Node {
		li().child(self.children).into_node()
	}
}
