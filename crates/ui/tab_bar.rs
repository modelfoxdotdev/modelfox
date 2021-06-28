use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TabBar {
	pub children: Vec<Node>,
}

impl Component for TabBar {
	fn into_node(self) -> Node {
		div().class("tab-bar").child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Tab {
	#[builder]
	pub selected: bool,
	#[builder]
	pub disabled: Option<bool>,
	pub children: Vec<Node>,
}

impl Component for Tab {
	fn into_node(self) -> Node {
		let selected = if self.selected {
			Some("tab-bar-tab-selected")
		} else {
			None
		};
		let disabled = if self.disabled.unwrap_or(false) {
			Some("tab-bar-tab-disabled")
		} else {
			None
		};
		div()
			.class("tab-bar-tab")
			.class(selected)
			.class(disabled)
			.child(self.children)
			.into_node()
	}
}

#[derive(builder, children, new)]
pub struct TabLink {
	pub href: String,
	pub selected: bool,
	#[builder]
	#[new(default)]
	pub disabled: Option<bool>,
	#[new(default)]
	pub children: Vec<Node>,
}

impl Component for TabLink {
	fn into_node(self) -> Node {
		Tab::new()
			.selected(self.selected)
			.disabled(self.disabled)
			.child(
				a().class("tab-bar-tab-link")
					.attribute("href", self.href)
					.child(self.children),
			)
			.into_node()
	}
}
