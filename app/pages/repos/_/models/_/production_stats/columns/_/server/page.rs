use html::{component, html, Props};
use tangram_app_common::{
	date_window::DateWindow, date_window_select_field::DateWindowSelectField,
};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;
use tangram_ui as ui;

pub use crate::enum_column::*;
pub use crate::number_column::*;
pub use crate::text_column::*;

#[derive(Props)]
pub struct PageProps {
	pub column_name: String,
	pub date_window: DateWindow,
	pub id: String,
	pub inner: Inner,
	pub model_layout_props: ModelLayoutProps,
}

pub enum Inner {
	Number(NumberColumnProps),
	Enum(EnumColumnProps),
	Text(TextColumnProps),
}

pub struct IntervalBoxChartDataPoint {
	pub label: String,
	pub stats: Option<IntervalBoxChartDataPointStats>,
}

pub struct IntervalBoxChartDataPointStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

pub struct OverallBoxChartData {
	pub production: Option<OverallBoxChartDataStats>,
	pub training: OverallBoxChartDataStats,
}

pub struct OverallBoxChartDataStats {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Number(inner) => html! { <NumberColumn {inner} /> },
		Inner::Enum(inner) => html! { <EnumColumn {inner} /> },
		Inner::Text(inner) => html! { <TextColumn {inner} /> },
	};
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{props.column_name}</ui::H1>
					<DateWindowSelectForm date_window={props.date_window} />
					{inner}
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct DateWindowSelectFormProps {
	date_window: DateWindow,
}

#[component]
fn DateWindowSelectForm(props: DateWindowSelectFormProps) {
	html! {
		<ui::Form>
			<DateWindowSelectField date_window={props.date_window} />
			<noscript>
				<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
					{"Submit"}
				</ui::Button>
			</noscript>
		</ui::Form>
	}
}
