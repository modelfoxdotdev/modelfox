use pinwheel::prelude::*;
use tangram_www_benchmarks_common::Benchmarks;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(ComponentBuilder)]
pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_www_benchmarks_client")
			.child(PageLayout::new().child(Dehydrate::new("benchmarks", Benchmarks::new())))
			.into_node()
	}
}
