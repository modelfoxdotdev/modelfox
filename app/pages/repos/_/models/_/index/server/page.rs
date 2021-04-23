use crate::{
	binary_classifier::binary_classifier_index_page,
	multiclass_classifier::multiclass_classifier_index_page, regressor::regressor_index_page,
};
use html::{component, html, Props};
use tangram_app_layouts::document::{Document, DocumentProps};
use tangram_app_layouts::model_layout::{ModelLayout, ModelLayoutProps};
use tangram_serve::client;

pub use {
	crate::binary_classifier::{BinaryClassifierMetricsSectionProps, BinaryClassifierProps},
	crate::multiclass_classifier::{
		MulticlassClassifierClassMetrics, MulticlassClassifierMetricsSectionProps,
		MulticlassClassifierProps,
	},
	crate::regressor::{RegressorMetricsSectionProps, RegressorProps},
};

#[derive(Props)]
pub struct PageProps {
	pub id: String,
	pub inner: Inner,
	pub model_layout_props: ModelLayoutProps,
}

pub enum Inner {
	Regressor(RegressorProps),
	BinaryClassifier(BinaryClassifierProps),
	MulticlassClassifier(MulticlassClassifierProps),
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Regressor(inner) => regressor_index_page(inner),
		Inner::BinaryClassifier(inner) => binary_classifier_index_page(inner),
		Inner::MulticlassClassifier(inner) => multiclass_classifier_index_page(inner),
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
