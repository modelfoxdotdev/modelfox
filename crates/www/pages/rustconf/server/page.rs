use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let iframe = iframe().attribute("src","https://docs.google.com/presentation/d/e/2PACX-1vSkASvl7qMtfKgwR5FUwb9dX0casToUSgVxHPjsKQMRyMSMGdzGJLwGyEef_y68ti5VkgC08YB1_iSp/embed?start=false&loop=false&delayms=3000").attribute("frameborder", "0").attribute("width","960").attribute("height","569").attribute("allowfullscreen","true").attribute("webkitallowfullscreen","true").attribute("mozallowfullscreen","true");
		let content = div()
			.style("display", "grid")
			.style("justify-items", "center")
			.child(ui::S1::new().child(iframe));
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}
