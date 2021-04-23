use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;

pub use crate::binary_classifier::*;
pub use crate::multiclass_classifier::*;
pub use crate::regressor::*;

#[derive(Props)]
pub struct PageProps {
	pub id: String,
	pub inner: Inner,
	pub model_layout_props: ModelLayoutProps,
}

pub enum Inner {
	Regressor(RegressorProductionMetricsProps),
	BinaryClassifier(BinaryClassifierProductionMetricsProps),
	MulticlassClassifier(MulticlassClassifierProductionMetricsProps),
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

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Regressor(inner) => html! {
			<RegressorProductionMetrics {inner} />
		},
		Inner::BinaryClassifier(inner) => html! {
			<BinaryClassifierProductionMetrics {inner} />
		},
		Inner::MulticlassClassifier(inner) => html! {
			<MulticlassClassifierProductionMetrics {inner} />
		},
	};
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				{inner}
			</ModelLayout>
		</Document>
	}
}
