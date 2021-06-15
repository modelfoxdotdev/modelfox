use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};

pub use crate::binary_classifier::*;
pub use crate::multiclass_classifier::*;
pub use crate::regressor::*;

#[derive(ComponentBuilder)]
pub struct Page {
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Regressor(RegressorProductionMetrics),
	BinaryClassifier(BinaryClassifierProductionMetrics),
	MulticlassClassifier(MulticlassClassifierProductionMetrics),
}

pub struct TrueValuesCountChartEntry {
	pub label: String,
	pub count: u64,
}

pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

pub struct AccuracyChart {
	pub data: Vec<AccuracyChartEntry>,
	pub training_accuracy: f32,
}

pub struct AccuracyChartEntry {
	pub accuracy: Option<f32>,
	pub label: String,
}

pub struct ClassMetricsTableEntry {
	pub class_name: String,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Regressor(inner) => inner.into_node(),
			Inner::BinaryClassifier(inner) => inner.into_node(),
			Inner::MulticlassClassifier(inner) => inner.into_node(),
		};
		Document::new()
			.client("tangram_app_production_metrics_index_client")
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
