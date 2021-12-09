pub use crate::{binary_classifier::*, multiclass_classifier::*, regressor::*};
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};

pub struct Page {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Regressor(inner) => inner.into_node(),
			Inner::BinaryClassifier(inner) => inner.into_node(),
			Inner::MulticlassClassifier(inner) => inner.into_node(),
		};
		Document::new()
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
