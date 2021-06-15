use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct MetricsRow {
	#[children]
	pub children: Vec<Node>,
}

impl Component for MetricsRow {
	fn into_node(self) -> Node {
		div().class("metrics-row").child(self.children).into_node()
	}
}
