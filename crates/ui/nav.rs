use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Nav {
	#[builder]
	title: Option<String>,
	pub children: Vec<Node>,
}

impl Component for Nav {
	fn into_node(self) -> Node {
		div()
			.child(
				details()
					.class("nav-details")
					.child(summary().child(self.title)),
			)
			.child(nav().class("nav").child(self.children))
			.into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct NavItem {
	#[builder]
	pub title: Option<String>,
	#[builder]
	pub href: Option<String>,
	#[builder]
	pub selected: Option<bool>,
	pub children: Vec<Node>,
}

impl Component for NavItem {
	fn into_node(self) -> Node {
		let selected = self.selected.unwrap_or(false);
		let selected_class = if selected {
			Some("nav-item-selected")
		} else {
			None
		};
		let clickable_class = if self.href.is_some() {
			Some("nav-item-clickable")
		} else {
			None
		};
		div()
			.class("nav-item")
			.class(selected_class)
			.class(clickable_class)
			.child(a().attribute("href", self.href).child(self.title))
			.child(self.children)
			.into_node()
	}
}

#[derive(builder, children, new)]
pub struct NavSection {
	pub title: String,
	#[new(default)]
	pub children: Vec<Node>,
}

impl Component for NavSection {
	fn into_node(self) -> Node {
		div()
			.class("nav-section")
			.child(div().class("nav-section-title").child(self.title))
			.child(self.children)
			.into_node()
	}
}
