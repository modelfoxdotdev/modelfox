use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_content::{BlogPost, Content};
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(ComponentBuilder)]
pub struct Page {
	slug: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let blog_post = BlogPost::from_slug(self.slug).unwrap();
		let heading = div()
			.child(ui::H1::new().child(blog_post.front_matter.title))
			.child(
				div()
					.class("blog-post-date")
					.child(blog_post.front_matter.date),
			);
		Document::new()
			.child(
				PageLayout::new().child(
					ui::S1::new()
						.child(heading)
						.child(ui::Markdown::new(blog_post.markdown)),
				),
			)
			.into_node()
	}
}
