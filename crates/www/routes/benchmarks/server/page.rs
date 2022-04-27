use modelfox_www_benchmarks_common::Benchmarks;
use modelfox_www_layouts::{document::Document, page_layout::PageLayout};
use pinwheel::prelude::*;

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("modelfox_www_benchmarks_client")
			.child(PageLayout::new().child(Dehydrate::new("benchmarks", Benchmarks::new())))
			.into_node()
	}
}
