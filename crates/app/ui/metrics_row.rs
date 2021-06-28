use pinwheel::prelude::*;

#[derive(children, Default, new)]
#[new(default)]
pub struct MetricsRow {
	pub children: Vec<Node>,
}

impl Component for MetricsRow {
	fn into_node(self) -> Node {
		div().class("metrics-row").child(self.children).into_node()
	}
}
