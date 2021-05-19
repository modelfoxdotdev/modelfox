use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, TrainPage},
	document::Document,
};

#[derive(ComponentBuilder)]
pub struct Page {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let p = ui::P::new()
			.child("If you want more control over training you can provide a json config file to the tangram train command: ")
			.child(ui::InlineCode::new().child("--config config.json"))
			.child(".")
			.child(" Below is an example config file. It includes all of the possible options you can set. Every field is optional.");
		let code = ui::highlight(
			include_str!("./heart_disease.json"),
			ui::Language::Javascript,
		);
		let window = ui::Window::new().child(
			ui::Code::new()
				.code(Cow::Owned(code))
				.hide_line_numbers(Some(false)),
		);
		let content = ui::S1::new()
			.child(ui::H1::new().child("Configuration"))
			.child(ui::S2::new().child(p).child(window));
		let layout =
			DocsLayout::new(DocsPage::Train(TrainPage::Configuration), None).child(content);
		Document::new().child(layout).into_node()
	}
}
