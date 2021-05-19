use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::Document,
};

#[derive(ComponentBuilder)]
pub struct Page {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let list = ui::UnorderedList::new().child(ui::ListItem::new().child("Train a model with the Tangram CLI to predict whether cardiac patients have heart disease.")).child(ui::ListItem::new().child("Make predictions using the Tangram language libraries.")).child(ui::ListItem::new().child("Learn more about our model with the Tangram web app.")).child(ui::ListItem::new().child("Set up production monitoring and debug our model's performance."));
		let prev_next_buttons = div().class("docs-prev-next-buttons").child(div()).child(
			ui::Link::new()
				.href("train".to_owned())
				.child("Next: Train a Model. >"),
		);
		let content = ui::S1::new()
			.child(ui::H1::new().child("Getting Started"))
			.child(
				ui::S2::new()
					.child(ui::P::new().child("Thanks for trying Tangram!"))
					.child(ui::P::new().child("In this getting started guide, we will:"))
					.child(list),
			)
			.child(prev_next_buttons);
		let layout = DocsLayout::new(DocsPage::GettingStarted(GettingStartedPage::Index), None)
			.child(content);
		Document::new().child(layout).into_node()
	}
}
