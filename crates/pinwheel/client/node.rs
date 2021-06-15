use super::{
	element::Element, fragment::Fragment, signal::SignalNode, signal_vec::SignalVecNode, text::Text,
};
use wasm_bindgen::JsCast;
use web_sys as dom;

pub enum Node {
	Text(Text),
	Element(Element),
	Fragment(Fragment),
	Signal(SignalNode),
	SignalVec(SignalVecNode),
}

impl std::fmt::Display for Node {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unimplemented!()
	}
}

impl Node {
	pub(crate) fn first_dom_node(&self) -> &dom::Node {
		match self {
			Node::Text(text) => text.text.unchecked_ref(),
			Node::Element(element) => element.element.unchecked_ref(),
			Node::Fragment(fragment) => fragment.start_marker().unchecked_ref(),
			Node::Signal(signal_node) => signal_node.marker.unchecked_ref(),
			Node::SignalVec(signal_vec_node) => signal_vec_node.start_marker.unchecked_ref(),
		}
	}

	pub(crate) fn insert_before(
		&mut self,
		parent_dom_node: &dom::Node,
		reference_dom_node: Option<&dom::Node>,
	) {
		match self {
			Node::Text(text) => {
				parent_dom_node
					.insert_before(text.text.unchecked_ref(), reference_dom_node)
					.unwrap();
			}
			Node::Element(element) => {
				parent_dom_node
					.insert_before(element.element.unchecked_ref(), reference_dom_node)
					.unwrap();
			}
			Node::Fragment(fragment) => {
				parent_dom_node
					.insert_before(&fragment.start_marker(), reference_dom_node)
					.unwrap();
				let (dom_fragment, start_marker, children) = match fragment {
					Fragment::Detached {
						dom_fragment,
						start_marker,
						children,
					} => (dom_fragment, start_marker, children),
					Fragment::Attached { .. } => panic!(),
				};
				parent_dom_node
					.insert_before(&dom_fragment, reference_dom_node)
					.unwrap();
				*fragment = Fragment::Attached {
					start_marker: start_marker.take().unwrap(),
					children: children.take().unwrap(),
				};
			}
			Node::Signal(signal_node) => {
				if let Some(child) = signal_node.child.borrow_mut().as_mut() {
					child.insert_before(parent_dom_node, reference_dom_node)
				}
				parent_dom_node
					.insert_before(&signal_node.marker, reference_dom_node)
					.unwrap();
			}
			Node::SignalVec(signal_vec_node) => {
				parent_dom_node
					.insert_before(&signal_vec_node.start_marker, reference_dom_node)
					.unwrap();
				for child in signal_vec_node.children.borrow_mut().iter_mut() {
					child.insert_before(parent_dom_node, reference_dom_node);
				}
				parent_dom_node
					.insert_before(&signal_vec_node.end_marker, reference_dom_node)
					.unwrap();
			}
		}
	}

	pub(crate) fn remove(&mut self) {
		match self {
			Node::Text(text) => {
				text.text.remove();
			}
			Node::Element(element) => {
				element.element.remove();
			}
			Node::Fragment(fragment) => {
				let (start_marker, children) = match fragment {
					Fragment::Detached { .. } => panic!(),
					Fragment::Attached {
						start_marker,
						children,
					} => (start_marker, children),
				};
				start_marker.remove();
				for node in children.iter_mut() {
					node.remove();
				}
			}
			Node::Signal(signal_node) => {
				if let Some(child) = signal_node.child.borrow_mut().as_mut() {
					child.remove();
				}
				signal_node.marker.remove();
			}
			Node::SignalVec(signal_vec_node) => {
				signal_vec_node.start_marker.remove();
				for child in signal_vec_node.children.borrow_mut().iter_mut() {
					child.remove();
				}
				signal_vec_node.end_marker.remove();
			}
		};
	}
}
