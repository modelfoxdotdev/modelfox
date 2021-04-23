use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	predict::{PredictOutput, PredictOutputProps},
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken},
};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::{BarChart, BoxChart},
};
use tangram_core::predict::PredictInputValue;
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub model_layout_props: ModelLayoutProps,
	pub inner: Inner,
}

pub enum Inner {
	Form(FormProps),
	Output(PredictOutputProps),
}

#[derive(Props)]
pub struct FormProps {
	pub fields: Vec<FieldProps>,
}

pub enum FieldProps {
	Unknown(UnknownFieldProps),
	Number(NumberFieldProps),
	Enum(EnumFieldProps),
	Text(TextFieldProps),
}

pub enum FieldValue {
	Number(f64),
	String(String),
}

impl From<PredictInputValue> for FieldValue {
	fn from(value: PredictInputValue) -> Self {
		match value {
			PredictInputValue::Number(value) => FieldValue::Number(value),
			PredictInputValue::String(value) => FieldValue::String(value),
		}
	}
}

impl std::fmt::Display for FieldValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FieldValue::Number(n) => write!(f, "{}", n.to_string()),
			FieldValue::String(s) => write!(f, "{}", s),
		}
	}
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.inner {
		Inner::Form(inner) => {
			html! {
				<Form {inner} />
			}
		}
		Inner::Output(inner) => {
			html! {
				<PredictOutput {inner} />
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
					<ui::H1>{"Playground"}</ui::H1>
					{inner}
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[component]
fn Form(props: FormProps) {
	let fields = props
		.fields
		.into_iter()
		.map(|field_props| match field_props {
			FieldProps::Unknown(field_props) => {
				html! { <UnknownField {field_props} /> }
			}
			FieldProps::Number(field_props) => {
				html! { <NumberField {field_props} /> }
			}
			FieldProps::Enum(field_props) => {
				html! { <EnumField {field_props} /> }
			}
			FieldProps::Text(field_props) => {
				html! { <TextField {field_props} /> }
			}
		})
		.collect::<Vec<_>>();
	html! {
		<ui::Form>
			<div class="predict-form-grid">
				{fields}
			</div>
			<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
				{"Predict"}
			</ui::Button>
		</ui::Form>
	}
}

#[derive(Props)]
pub struct UnknownFieldProps {
	pub name: String,
	pub value: FieldValue,
}

#[component]
fn UnknownField(props: UnknownFieldProps) {
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<UnknownColumnToken />
				</div>
				<ui::TextField
					label?={Some(props.name.clone())}
					name?={Some(props.name)}
					value?={Some(props.value.to_string())}
				/>
			</div>
			<div></div>
		</>
	}
}

#[derive(Props)]
pub struct NumberFieldProps {
	pub name: String,
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub value: FieldValue,
}

#[component]
fn NumberField(props: NumberFieldProps) {
	let column_chart_series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: vec![BoxChartPoint {
			label: props.name.to_owned(),
			x: 0.0,
			y: Some(BoxChartValue {
				min: props.min.to_f64().unwrap(),
				max: props.max.to_f64().unwrap(),
				p25: props.p25.to_f64().unwrap(),
				p50: props.p50.to_f64().unwrap(),
				p75: props.p75.to_f64().unwrap(),
			}),
		}],
		title: Some("quartiles".to_owned()),
	}];
	let value = props.value.to_string();
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<NumberColumnToken />
				</div>
				<ui::TextField
					label?={Some(props.name.clone())}
					name?={Some(props.name.clone())}
					value?={Some(value)}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BoxChart
					class?="column-chart"
					hide_legend?={Some(true)}
					id?={Some(props.name)}
					series?={Some(column_chart_series)}
					should_draw_x_axis_labels?={Some(false)}
					should_draw_y_axis_labels?={Some(false)}
				/>
			</div>
		</>
	}
}

#[derive(Props)]
pub struct EnumFieldProps {
	pub name: String,
	pub options: Vec<String>,
	pub value: FieldValue,
	pub histogram: Vec<(String, u64)>,
}

#[component]
fn EnumField(props: EnumFieldProps) {
	let series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: props
			.histogram
			.iter()
			.enumerate()
			.map(|(index, (label, value))| BarChartPoint {
				label: label.to_owned(),
				x: index.to_f64().unwrap(),
				y: Some(value.to_f64().unwrap()),
			})
			.collect::<Vec<_>>(),
		title: Some("histogram".to_owned()),
	}];
	let options = props
		.options
		.iter()
		.map(|option| ui::SelectFieldOption {
			text: option.to_owned(),
			value: option.to_owned(),
		})
		.chain(std::iter::once(ui::SelectFieldOption {
			text: "".to_owned(),
			value: "".to_owned(),
		}))
		.collect::<Vec<_>>();
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<EnumColumnToken />
				</div>
				<ui::SelectField
					label?={Some(props.name.clone())}
					name?={Some(props.name.clone())}
					options?={Some(options)}
					value?={Some(props.value.to_string())}
				/>
			</div>
			<div class="predict-column-chart-wrapper">
				<BarChart
					class?="column-chart"
					hide_legend?={Some(true)}
					id?={Some(props.name)}
					series?={Some(series)}
					should_draw_x_axis_labels?={Some(false)}
					should_draw_y_axis_labels?={Some(false)}
				/>
			</div>
		</>
	}
}

#[derive(Props)]
pub struct TextFieldProps {
	pub name: String,
	pub value: FieldValue,
}

#[component]
fn TextField(props: TextFieldProps) {
	html! {
		<>
			<div class="predict-field-wrapper">
				<div>
					<TextColumnToken />
				</div>
				<ui::TextField
					label?={Some(props.name.clone())}
					name?={Some(props.name)}
					value?={Some(props.value.to_string())}
				/>
			</div>
			<div></div>
		</>
	}
}
