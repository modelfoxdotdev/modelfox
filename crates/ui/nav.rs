use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Nav {
	#[optional]
	title: Option<String>,
	#[children]
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

#[derive(ComponentBuilder)]
pub struct NavItem {
	#[optional]
	pub title: Option<String>,
	#[optional]
	pub href: Option<String>,
	#[optional]
	pub selected: Option<bool>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for NavItem {
	fn into_node(self) -> Node {
		let selected = self.selected.unwrap_or(false);
		let class = classes!(
			"nav-item",
			if selected {
				Some("nav-item-selected")
			} else {
				None
			},
			if self.href.is_some() {
				Some("nav-item-clickable")
			} else {
				None
			}
		);
		div()
			.attribute("class", class)
			.child(a().attribute("href", self.href).child(self.title))
			.child(self.children)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct NavSection {
	pub title: String,
	#[children]
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
