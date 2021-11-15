use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_content::{BlogPost, Content};
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let blog_posts = BlogPost::list().unwrap().into_iter().map(|blog_post| {
			let href = format!("/blog/{}/", blog_post.slug);
			div()
				.child(
					ui::Link::new()
						.href(href)
						.child(blog_post.front_matter.title),
				)
				.child(ui::P::new().child(blog_post.front_matter.date))
		});
		Document::new()
			.child(
				PageLayout::new().child(
					ui::S1::new()
						.child(ui::H1::new().child("Blog"))
						.child(ui::S2::new().children(blog_posts)),
				),
			)
			.into_node()
	}
}
