use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_content::{Content, DocsGuide};
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::Document,
};

#[derive(new)]
pub struct Page {
	pub slug: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let guide = DocsGuide::from_slug(self.slug).unwrap();
		let content = ui::S1::new()
			.child(ui::H1::new().child(guide.front_matter.title))
			.child(ui::Markdown::new(guide.markdown.into()));
		let layout = DocsLayout::new()
			.selected_page(DocsPage::Guides(guide.slug))
			.child(content);
		Document::new().child(layout).into_node()
	}
}
