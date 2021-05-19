use pinwheel::prelude::*;
use std::path::PathBuf;
use tangram_ui as ui;
use tangram_www_blog::BlogPost;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(ComponentBuilder)]
pub struct Page {
	blog_post_path: PathBuf,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let blog_post = BlogPost::from_path(&self.blog_post_path).unwrap();
		Document::new()
			.child(
				PageLayout::new().child(
					ui::S1::new()
						.child(
							div()
								.child(ui::H1::new().child(blog_post.title))
								.child(div().class("blog-post-date").child(blog_post.date)),
						)
						.child(ui::Markdown::new(blog_post.markdown)),
				),
			)
			.into_node()
	}
}
