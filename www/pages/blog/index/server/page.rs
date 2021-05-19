use pinwheel::prelude::*;
use std::path::PathBuf;
use tangram_ui as ui;
use tangram_www_blog::{blog_post_slugs_and_paths, BlogPost, BlogPostSlugAndPath};
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(ComponentBuilder)]
pub struct Page {
	blog_posts_path: PathBuf,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let blog_posts = blog_post_slugs_and_paths(&self.blog_posts_path)
			.unwrap()
			.into_iter()
			.map(|slug_and_path| {
				let BlogPostSlugAndPath { path, .. } = slug_and_path;
				BlogPost::from_path(&path).unwrap()
			})
			.map(|blog_post| {
				let href = format!("/blog/{}", blog_post.slug);
				div()
					.child(ui::Link::new().href(href).child(blog_post.title))
					.child(ui::P::new().child(blog_post.date))
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
