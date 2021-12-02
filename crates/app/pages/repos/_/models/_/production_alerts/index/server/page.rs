use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub model_id: String,
	pub model_layout_info: ModelLayoutInfo,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(ModelLayout::new(self.model_layout_info))
			.into_node()
	}
}
