pub use crate::binary_classifier::*;
pub use crate::common::{
	ClassifierChartEntry, PredictionCountChartEntry, ProductionTrainingHistogram,
};
pub use crate::multiclass_classifier::*;
pub use crate::regressor::*;
use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;

#[derive(Props)]
pub struct PageProps {
	pub model_id: String,
	pub model_layout_props: ModelLayoutProps,
	pub inner: InnerProps,
}

pub enum InnerProps {
	Regressor(RegressorProps),
	BinaryClassifier(BinaryClassifierProps),
	MulticlassClassifier(MulticlassClassifierProps),
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	let inner = match props.inner {
		InnerProps::Regressor(props) => html! {<RegressorPage {props} />},
		InnerProps::BinaryClassifier(props) => html! {<BinaryClassifierPage {props} />},
		InnerProps::MulticlassClassifier(props) => html! {<MulticlassClassifierPage {props} />},
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
			{inner}
			</ModelLayout>
		</Document>
	}
}
