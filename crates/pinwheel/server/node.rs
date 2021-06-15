use super::{Element, Fragment, SignalNode, SignalVecNode, Text};
use web_sys as dom;

pub enum Node {
	Text(Text),
	Element(Element),
	Fragment(Fragment),
	Signal(SignalNode),
	SignalVec(SignalVecNode),
}

impl Node {
	#[allow(dead_code)]
	pub(crate) fn first_dom_node(&self) -> &dom::Node {
		unimplemented!()
	}

	pub(crate) fn insert_before(
		&mut self,
		_parent_dom_node: &dom::Node,
		_reference_dom_node: Option<&dom::Node>,
	) {
		unimplemented!()
	}

	pub(crate) fn remove(&mut self) {
		unimplemented!()
	}
}

impl std::fmt::Display for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Node::Text(text) => {
				write!(f, "{}", text)?;
			}
			Node::Element(element) => {
				write!(f, "{}", element)?;
			}
			Node::Fragment(fragment) => {
				write!(f, "{}", fragment)?;
			}
			Node::Signal(signal) => {
				write!(f, "{}", signal)?;
			}
			Node::SignalVec(signal_vec) => {
				write!(f, "{}", signal_vec)?;
			}
		}
		Ok(())
	}
}
