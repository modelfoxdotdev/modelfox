use modelfox_ui as ui;
use modelfox_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::Document,
};
use pinwheel::prelude::*;

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let p1 = ui::P::new().child("Welcome to the documentation for ModelFox!");
		let p2 = ui::P::new().child("ModelFox makes it easy for programmers to train, deploy, and monitor machine learning models. With ModelFox, you:");
		let list = ui::UnorderedList::new()
			.child(ui::ListItem::new().child("Train a model from a CSV file on the command line."))
			.child(
				ui::ListItem::new().child(
					"Make predictions from Elixir, Go, JavaScript, PHP, Python, Ruby, or Rust.",
				),
			)
			.child(ui::ListItem::new().child(
				"Learn about your models and monitor them in production from your browser.",
			));
		let content = ui::S1::new()
			.child(ui::H1::new("Overview"))
			.child(ui::S2::new().child(p1).child(p2).child(list));
		let layout = DocsLayout::new()
			.selected_page(DocsPage::Overview)
			.child(content);
		Document::new().child(layout).into_node()
	}
}
