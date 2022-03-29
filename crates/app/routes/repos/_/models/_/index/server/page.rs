pub use crate::{
	binary_classifier::{BinaryClassifier, BinaryClassifierMetricsSection},
	multiclass_classifier::{
		MulticlassClassifier, MulticlassClassifierClassMetrics, MulticlassClassifierMetricsSection,
	},
	regressor::{Regressor, RegressorMetricsSection},
};
use pinwheel::prelude::*;
use modelfox_app_layouts::document::Document;
use modelfox_app_layouts::model_layout::{ModelLayout, ModelLayoutInfo};

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
			.client("modelfox_app_model_index_client")
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
