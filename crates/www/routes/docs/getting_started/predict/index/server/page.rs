use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				DocsLayout::new()
					.selected_page(DocsPage::GettingStarted(GettingStartedPage::Predict(
						PredictPage::Index,
					)))
					.child(
						ui::S1::new().child(ui::H1::new("Predict")).child(
							ui::S2::new().child(
								ui::UnorderedList::new()
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("cli".to_owned())
												.title("CLI".to_owned())
												.child("CLI"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("elixir".to_owned())
												.title("Elixir".to_owned())
												.child("Elixir"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("go".to_owned())
												.title("Go".to_owned())
												.child("Go"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("javascript".to_owned())
												.title("JavaScript".to_owned())
												.child("JavaScript"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("php".to_owned())
												.title("PHP".to_owned())
												.child("PHP"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("python".to_owned())
												.title("Python".to_owned())
												.child("Python"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("ruby".to_owned())
												.title("Ruby".to_owned())
												.child("Ruby"),
										),
									)
									.child(
										ui::ListItem::new().child(
											ui::Link::new()
												.href("rust".to_owned())
												.title("Rust".to_owned())
												.child("Rust"),
										),
									),
							),
						),
					),
			)
			.into_node()
	}
}
