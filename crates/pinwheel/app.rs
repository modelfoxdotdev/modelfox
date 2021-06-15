use super::Node;
use wasm_bindgen::JsCast;
use web_sys as dom;

pub struct App {
	node: Node,
}

impl App {
	pub fn new(dom_node: dom::Node, node: impl Into<Node>) -> App {
		let mut node = node.into();
		dom_node.unchecked_ref::<dom::Element>().set_inner_html("");
		node.insert_before(&dom_node, None);
		App { node }
	}

	pub fn forget(self) {
		std::mem::forget(self)
	}
}

impl Drop for App {
	fn drop(&mut self) {
		self.node.remove();
	}
}
