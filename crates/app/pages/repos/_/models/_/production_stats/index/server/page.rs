pub use crate::{
	binary_classifier::*,
	common::{ClassifierChartEntry, PredictionCountChartEntry, ProductionTrainingHistogram},
	multiclass_classifier::*,
	regressor::*,
};
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};

#[derive(ComponentBuilder)]
pub struct Page {
	pub model_id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub inner: Inner,
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
			.client("tangram_app_production_stats_index_client")
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
