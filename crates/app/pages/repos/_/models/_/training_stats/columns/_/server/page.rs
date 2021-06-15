pub use crate::{enum_column::*, number_column::*, text_column::*};
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};

#[derive(ComponentBuilder)]
pub struct Page {
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Number(NumberColumn),
	Enum(EnumColumn),
	Text(TextColumn),
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Number(inner) => inner.into_node(),
			Inner::Enum(inner) => inner.into_node(),
			Inner::Text(inner) => inner.into_node(),
		};
		Document::new()
			.client("tangram_app_training_stats_column_client")
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
