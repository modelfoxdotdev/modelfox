use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let title = ui::H1::new().child("Jobs");
		let job1 = ui::S2::new().child(ui::H2::new().child("Rust Developer")).child(ui::P::new().child("Do you love Rust, Machine Learning, and writing really fast and memory efficient code? At Tangram, you'll get to work on everything from our core machine learning algorithms to writing front-end code in Rust! We are looking for developers with experience in Rust and familiarity with (or willingness to learn) machine learning concepts. If this sounds exciting, let's talk! Email us at ").child(ui::Link::new().href("mailto:jobs@tangram.dev".to_string()).title("jobs@tangram.dev".to_string()).child("jobs@tangram.dev")));
		let job2 = ui::S2::new()
			.child(ui::H2::new().child("Machine Learning Developer Advocate"))
			.child(ui::P::new().child("Do you love writing blog posts about machine learning and developer tools? Do you want to get on calls with developers and demo Tangram and figure out how to help them be successful deploying machine learning models? If this sounds exciting, let's talk! Email us at ").child(ui::Link::new().href("mailto:jobs@tangram.dev".to_string()).title("jobs@tangram.dev".to_string()).child("jobs@tangram.dev")));
		let content = div()
			.style("display", "grid")
			.child(ui::S1::new().child(title).child(job1).child(job2));
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}
