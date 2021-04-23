pub use crate::{enum_column::*, number_column::*, text_column::*};
use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;

#[derive(Props)]
pub struct PageProps {
	pub inner: Inner,
	pub model_layout_props: ModelLayoutProps,
}

pub enum Inner {
	Number(NumberColumnProps),
	Enum(EnumColumnProps),
	Text(TextColumnProps),
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Number(inner) => html! {
			<NumberColumn {inner} />
		},
		Inner::Enum(inner) => html! {
			<EnumColumn {inner} />
		},
		Inner::Text(inner) => html! {
			<TextColumn {inner} />
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
