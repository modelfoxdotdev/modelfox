use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage, PrevNextButtons},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				DocsLayout::new()
					.selected_page(DocsPage::GettingStarted(GettingStartedPage::Predict(
						PredictPage::CLI,
					)))
					.child(
						ui::S1::new()
							.child(ui::H1::new().child("Predict with the CLI"))
							.child(ui::Markdown::new(include_str!("./page.md").into()))
							.child(
								PrevNextButtons::new()
									.prev("../train", "Train a model.")
									.next("../inspect", "Inspect your model."),
							),
					),
			)
			.into_node()
	}
}
