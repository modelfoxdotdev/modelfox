use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::Document,
};

#[derive(ComponentBuilder)]
pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let p1 = ui::P::new().child("Welcome to the documentation for Tangram!");
		let p2 = ui::P::new().child("Tangram is an automated machine learning framework designed for programmers. With Tangram, you:");
		let list = ui::UnorderedList::new()
			.child(ui::ListItem::new().child("Train a model from a CSV on the command line."))
			.child(
				ui::ListItem::new()
					.child("Make predictions from Elixir, Go, JavaScript, Python, Ruby, or Rust."),
			)
			.child(ui::ListItem::new().child(
				"Learn about your models and monitor them in production from your browser.",
			));
		let content = ui::S1::new()
			.child(ui::H1::new().child("Overview"))
			.child(ui::S2::new().child(p1).child(p2).child(list));
		let layout = DocsLayout::new(DocsPage::Overview, None).child(content);
		Document::new().child(layout).into_node()
	}
}
