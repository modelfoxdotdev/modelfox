use super::{node::Node, Handle};
use futures::future::{abortable, ready, FutureExt};
use futures_signals::signal::{Signal, SignalExt};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys as dom;

pub struct SignalNode {
	pub(crate) marker: dom::Comment,
	pub(crate) child: Rc<RefCell<Option<Node>>>,
	#[allow(dead_code)]
	pub(crate) handle: Option<Handle>,
}

impl SignalNode {
	pub fn new<T, S>(signal: S) -> SignalNode
	where
		T: Into<Node>,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let document = dom::window().unwrap().document().unwrap();
		let marker = document.create_comment("");
		let child: Rc<RefCell<Option<Node>>> = Rc::new(RefCell::new(None));
		let (future, handle) = abortable(signal.for_each({
			let marker = marker.clone();
			let child = child.clone();
			move |node| {
				let mut node = node.into();
				let mut child = child.borrow_mut();
				if let Some(child) = &mut *child {
					child.remove();
				}
				let parent_dom_node = marker.parent_node().unwrap();
				node.insert_before(&parent_dom_node, Some(marker.unchecked_ref()));
				*child = Some(node);
				ready(())
			}
		}));
		spawn_local(future.map(|_| ()));
		let handle = Handle(handle);
		SignalNode {
			child,
			marker,
			handle: Some(handle),
		}
	}
}
