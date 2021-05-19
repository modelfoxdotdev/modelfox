use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct FieldLabel {
	pub html_for: Option<String>,
	#[children]
	children: Vec<Node>,
}

impl Component for FieldLabel {
	fn into_node(self) -> Node {
		label()
			.class("field-label")
			.attribute("for", self.html_for)
			.child(self.children)
			.into_node()
	}
}
