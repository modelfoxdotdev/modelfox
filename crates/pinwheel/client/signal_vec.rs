use super::{node::Node, Handle};
use futures::future::{abortable, ready, FutureExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt, VecDiff};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use web_sys as dom;

pub struct SignalVecNode {
	pub(crate) start_marker: dom::Comment,
	pub(crate) end_marker: dom::Comment,
	pub(crate) children: Rc<RefCell<Vec<Node>>>,
	#[allow(dead_code)]
	pub(crate) handle: Option<Handle>,
}

impl SignalVecNode {
	pub fn new<T, S>(signal_vec: S) -> SignalVecNode
	where
		T: Into<Node>,
		S: 'static + Unpin + SignalVec<Item = T>,
	{
		let document = dom::window().unwrap().document().unwrap();
		let start_marker = document.create_comment("");
		let end_marker = document.create_comment("");
		let children: Rc<RefCell<Vec<Node>>> = Rc::new(RefCell::new(Vec::new()));
		let (future, handle) = abortable(signal_vec.for_each({
			let end_marker = end_marker.clone();
			let children = children.clone();
			move |diff| {
				let parent_dom_node = end_marker.parent_node().unwrap();
				let mut children = children.borrow_mut();
				match diff {
					VecDiff::Replace { values: nodes } => {
						for mut node in children.drain(..) {
							node.remove();
						}
						for node in nodes {
							let mut node = node.into();
							node.insert_before(&parent_dom_node, Some(&end_marker));
							children.push(node);
						}
					}
					VecDiff::InsertAt { index, value: node } => {
						let mut node = node.into();
						let sibling_node = children.get(index).unwrap();
						node.insert_before(&parent_dom_node, Some(sibling_node.first_dom_node()));
						children.insert(index, node);
					}
					VecDiff::UpdateAt { index, value: node } => {
						let mut node = node.into();
						let sibling_node = children.get_mut(index).unwrap();
						node.insert_before(&parent_dom_node, Some(sibling_node.first_dom_node()));
						sibling_node.remove();
						*children.get_mut(index).unwrap() = node;
					}
					VecDiff::RemoveAt { index } => {
						let mut node = children.remove(index);
						node.remove();
					}
					VecDiff::Move {
						old_index,
						new_index,
					} => {
						let mut node = children.remove(old_index);
						node.insert_before(
							&parent_dom_node,
							Some(
								children
									.get(new_index)
									.map(|child| child.first_dom_node())
									.unwrap_or(&end_marker),
							),
						);
						children.insert(new_index, node);
					}
					VecDiff::Push { value: node } => {
						let mut node = node.into();
						node.insert_before(&parent_dom_node, Some(&end_marker));
						children.push(node);
					}
					VecDiff::Pop {} => {
						let mut node = children.pop().unwrap();
						node.remove();
					}
					VecDiff::Clear {} => {
						for mut node in children.drain(..) {
							node.remove()
						}
					}
				}
				ready(())
			}
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		SignalVecNode {
			start_marker,
			end_marker,
			children,
			handle: Some(handle),
		}
	}
}
