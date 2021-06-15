use crate::{Element, Fragment, Node, SignalNode, SignalVecNode, Text};
use std::borrow::Cow;

pub trait Component {
	fn into_node(self) -> Node;
}

impl<T> From<T> for Node
where
	T: Component,
{
	fn from(value: T) -> Self {
		value.into_node()
	}
}

impl Component for Text {
	fn into_node(self) -> Node {
		Node::Text(self)
	}
}

impl Component for Element {
	fn into_node(self) -> Node {
		Node::Element(self)
	}
}

impl Component for Fragment {
	fn into_node(self) -> Node {
		Node::Fragment(self)
	}
}

impl Component for SignalNode {
	fn into_node(self) -> Node {
		Node::Signal(self)
	}
}

impl Component for SignalVecNode {
	fn into_node(self) -> Node {
		Node::SignalVec(self)
	}
}

impl Component for Cow<'static, str> {
	fn into_node(self) -> Node {
		Node::Text(Text::new(self))
	}
}

impl Component for String {
	fn into_node(self) -> Node {
		Node::Text(Text::new(self))
	}
}

impl Component for &'static str {
	fn into_node(self) -> Node {
		Node::Text(Text::new(self))
	}
}

impl<T> Component for Option<T>
where
	T: Into<Node>,
{
	fn into_node(self) -> Node {
		self.map(|s| s.into())
			.unwrap_or_else(|| Node::Fragment(Fragment::new()))
	}
}

impl<T> Component for Vec<T>
where
	T: Into<Node>,
{
	fn into_node(self) -> Node {
		let mut fragment = Fragment::new();
		for child in self {
			fragment = fragment.child(child.into());
		}
		Node::Fragment(fragment)
	}
}
