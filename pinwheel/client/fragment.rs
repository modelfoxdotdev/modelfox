use super::node::Node;
use web_sys as dom;

pub fn fragment() -> Fragment {
	Fragment::new()
}

pub enum Fragment {
	Detached {
		dom_fragment: dom::DocumentFragment,
		start_marker: Option<dom::Comment>,
		children: Option<Vec<Node>>,
	},
	Attached {
		start_marker: dom::Comment,
		children: Vec<Node>,
	},
}

impl Fragment {
	pub fn new() -> Fragment {
		let document = dom::window().unwrap().document().unwrap();
		let dom_fragment = document.create_document_fragment();
		let start_marker = document.create_comment("");
		dom_fragment.append_child(&start_marker).unwrap();
		Fragment::Detached {
			dom_fragment,
			start_marker: Some(start_marker),
			children: Some(Vec::new()),
		}
	}

	pub fn child<T>(mut self, child: T) -> Fragment
	where
		T: Into<Node>,
	{
		let mut child = child.into();
		let (dom_fragment, children) = match &mut self {
			Fragment::Detached {
				dom_fragment,
				children,
				..
			} => (dom_fragment, children),
			Fragment::Attached { .. } => panic!(),
		};
		child.insert_before(dom_fragment, None);
		children.as_mut().unwrap().push(child);
		self
	}

	pub(crate) fn start_marker(&self) -> &dom::Comment {
		match self {
			Fragment::Detached { start_marker, .. } => start_marker.as_ref().unwrap(),
			Fragment::Attached { start_marker, .. } => start_marker,
		}
	}
}
