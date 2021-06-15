use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_tuning_common::Tuning;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub tuning: Option<Tuning>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.tuning {
			Some(tuning) => Dehydrate::new("tuning", tuning).into_node(),
			None => ui::S1::new()
				.child(ui::P::new().child("Tuning is not supported for this model."))
				.into_node(),
		};
		Document::new()
			.client("tangram_app_tuning_client")
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
