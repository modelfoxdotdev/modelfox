use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let title = ui::H1::new().child("Jobs");
		let job1 = ui::S2::new().child(ui::H2::new().child("Rust Developer (Remote)")).child(ui::P::new().child(ui::Markdown::new("Tangram makes it easy for programmers to train, deploy, and monitor machine learning models. With Tangram, developers can train models and make predictions on the command line or with libraries for languages including Elixir, Go, JS, PHP, Python, Ruby, and Rust, and learn about their models and monitor them in production from a web application. Watch the demo on the [homepage](https://www.tangram.dev). Tangram is open source and everything is written in Rust, from the core machine learning algorithms to the web application. Check it out on [GitHub](https://www.github.com/tangramdotdev/tangram). We are looking for programmers who love developer tools and are excited to build machine learning tools with Rust! If you are interested, email us at [jobs@tangram.dev](mailto:jobs@tangram.dev).".into())));
		let content = div()
			.style("display", "grid")
			.child(ui::S1::new().child(title).child(job1));
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}
