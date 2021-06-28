use pinwheel::prelude::*;

#[derive(builder)]
pub struct Details {
	pub options: Vec<DetailsOption>,
	pub summary: Option<String>,
}

pub struct DetailsOption {
	pub href: String,
	pub title: String,
}

impl Component for Details {
	fn into_node(self) -> Node {
		details()
			.class("details")
			.child(summary().child(self.summary))
			.child(
				div()
					.class("details-list")
					.children(self.options.into_iter().map(|option| {
						a().class("details-list-item")
							.attribute("href", option.href)
							.child(option.title)
					})),
			)
			.into_node()
	}
}
