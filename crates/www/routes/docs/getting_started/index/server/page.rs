use modelfox_ui as ui;
use modelfox_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PrevNextButtons},
	document::Document,
};
use pinwheel::prelude::*;

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let list = ui::UnorderedList::new()
			.child(ui::ListItem::new().child("Train a model with the ModelFox CLI to predict whether cardiac patients have heart disease."))
			.child(ui::ListItem::new().child("Make predictions using the ModelFox language libraries."))
			.child(ui::ListItem::new().child("Learn more about our model with the ModelFox web app."))
			.child(ui::ListItem::new().child("Set up production monitoring and debug our model's performance."));
		let prev_next_buttons = PrevNextButtons::new().next("train", "Train a model.");
		let content = ui::S1::new()
			.child(ui::H1::new("Getting Started"))
			.child(
				ui::S2::new()
					.child(ui::P::new().child("Thanks for trying ModelFox!"))
					.child(ui::P::new().child("In this getting started guide, we will:"))
					.child(list),
			)
			.child(prev_next_buttons);
		let layout = DocsLayout::new()
			.selected_page(DocsPage::GettingStarted(GettingStartedPage::Index))
			.child(content);
		Document::new().child(layout).into_node()
	}
}
