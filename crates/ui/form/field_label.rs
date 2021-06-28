use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct FieldLabel {
	#[builder]
	pub html_for: Option<String>,
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
