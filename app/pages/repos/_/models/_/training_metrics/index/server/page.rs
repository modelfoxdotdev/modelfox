pub use crate::{binary_classifier::*, multiclass_classifier::*, regressor::*};
use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
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
		Inner::Regressor(inner) => {
			html! { <Regressor {inner} /> }
		}
		Inner::BinaryClassifier(inner) => {
			html! { <BinaryClassifier {inner} /> }
		}
		Inner::MulticlassClassifier(inner) => {
			html! { <MulticlassClassifier {inner} /> }
		}
	};
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				{inner}
			</ModelLayout>
		</Document>
	}
}
