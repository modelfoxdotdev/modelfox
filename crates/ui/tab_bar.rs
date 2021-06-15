use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct TabBar {
	#[children]
	pub children: Vec<Node>,
}

impl Component for TabBar {
	fn into_node(self) -> Node {
		div().class("tab-bar").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct Tab {
	pub selected: bool,
	#[optional]
	pub disabled: Option<bool>,
	#[children]
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
		let class = classes!("tab-bar-tab", selected, disabled);
		div().class(class).child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TabLink {
	pub href: String,
	pub selected: bool,
	#[optional]
	pub disabled: Option<bool>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for TabLink {
	fn into_node(self) -> Node {
		Tab::new(self.selected)
			.disabled(self.disabled)
			.child(
				a().class("tab-bar-tab-link")
					.attribute("href", self.href)
					.child(self.children),
			)
			.into_node()
	}
}
