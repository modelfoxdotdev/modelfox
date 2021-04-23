use html::{component, html, Props};
use tangram_app_common::predict::{PredictOutput, PredictOutputProps};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub identifier: String,
	pub inner: Inner,
	pub model_layout_props: ModelLayoutProps,
}

pub enum Inner {
	NotFound(Box<NotFoundProps>),
	Found(Box<FoundProps>),
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::NotFound(inner) => {
			let inner = *inner;
			html! {
				<NotFound {inner} />
			}
		}
		Inner::Found(inner) => {
			let inner = *inner;
			html! {
				<Found {inner} />
			}
		}
	};
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Prediction"}</ui::H1>
					{inner}
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct NotFoundProps {
	pub identifier: String,
}

#[component]
fn NotFound(props: NotFoundProps) {
	html! {
		<ui::Alert level={ui::Level::Danger}>
			{"Prediction with identifier "}
			<b>{props.identifier}</b>
			{" not found."}
		</ui::Alert>
	}
}

#[derive(Props)]
pub struct FoundProps {
	pub date: String,
	pub identifier: String,
	pub predict_output_props: PredictOutputProps,
}

#[component]
fn Found(props: FoundProps) {
	html! {
		<>
			<PredictionTable identifier={props.identifier} date={props.date} />
			<PredictOutput {props.predict_output_props} />
		</>
	}
}

#[derive(Props)]
pub struct PredictionTableProps {
	pub date: String,
	pub identifier: String,
}

#[component]
fn PredictionTable(props: PredictionTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Identifier"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Date"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				<ui::TableRow>
					<ui::TableCell>
						{props.identifier}
					</ui::TableCell>
					<ui::TableCell>
						{props.date}
					</ui::TableCell>
				</ui::TableRow>
			</ui::TableBody>
		</ui::Table>
	}
}
