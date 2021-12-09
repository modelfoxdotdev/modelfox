use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_common::predict::PredictOutput;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_playground_common::ColumnChart;
use tangram_app_ui::tokens::{
	EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken,
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::{BarChart, BoxChart},
};
use tangram_core::predict::PredictInputValue;
use tangram_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub inner: Inner,
}

pub enum Inner {
	Form(Form),
	Output(PredictOutput),
}

pub struct Form {
	pub fields: Vec<Field>,
}

pub enum Field {
	Unknown(UnknownField),
	Number(NumberField),
	Enum(EnumField),
	Text(TextField),
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
			FieldValue::Number(n) => write!(f, "{}", n),
			FieldValue::String(s) => write!(f, "{}", s),
		}
	}
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Form(inner) => inner.into_node(),
			Inner::Output(inner) => inner.into_node(),
		};
		Document::new()
			.client("tangram_app_playground_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child("Playground"))
						.child(inner),
				),
			)
			.into_node()
	}
}

impl Component for Form {
	fn into_node(self) -> Node {
		let fields = self.fields.into_iter().map(|field| match field {
			Field::Unknown(field) => field.into_node(),
			Field::Number(field) => field.into_node(),
			Field::Enum(field) => field.into_node(),
			Field::Text(field) => field.into_node(),
		});
		ui::Form::new()
			.child(div().class("predict-form-grid").children(fields))
			.child(
				ui::Button::new()
					.button_type(ui::ButtonType::Submit)
					.child("Predict"),
			)
			.into_node()
	}
}

pub struct UnknownField {
	pub name: String,
	pub value: FieldValue,
}

impl Component for UnknownField {
	fn into_node(self) -> Node {
		fragment()
			.child(
				div()
					.class("predict-field-wrapper")
					.child(div().child(UnknownColumnToken))
					.child(
						ui::TextField::new()
							.label(self.name.clone())
							.name(self.name)
							.value(self.value.to_string()),
					),
			)
			.child(div())
			.into_node()
	}
}

pub struct NumberField {
	pub name: String,
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub value: FieldValue,
}

impl Component for NumberField {
	fn into_node(self) -> Node {
		let series = vec![BoxChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: vec![BoxChartPoint {
				label: self.name.to_owned(),
				x: 0.0,
				y: Some(BoxChartValue {
					min: self.min.to_f64().unwrap(),
					max: self.max.to_f64().unwrap(),
					p25: self.p25.to_f64().unwrap(),
					p50: self.p50.to_f64().unwrap(),
					p75: self.p75.to_f64().unwrap(),
				}),
			}],
			title: Some("quartiles".to_owned()),
		}];
		let value = self.value.to_string();
		fragment()
			.child(
				div()
					.class("predict-field-wrapper")
					.child(div().child(NumberColumnToken))
					.child(
						ui::TextField::new()
							.label(self.name.clone())
							.name(self.name.clone())
							.value(value),
					),
			)
			.child(Dehydrate::new(
				self.name,
				ColumnChart::Box(
					BoxChart::new()
						.hide_legend(true)
						.series(series)
						.should_draw_x_axis_labels(false)
						.should_draw_y_axis_labels(false),
				),
			))
			.into_node()
	}
}

pub struct EnumField {
	pub name: String,
	pub options: Vec<String>,
	pub value: FieldValue,
	pub histogram: Vec<(String, u64)>,
}

impl Component for EnumField {
	fn into_node(self) -> Node {
		let series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: self
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
		let options = self
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
		fragment()
			.child(
				div()
					.class("predict-field-wrapper")
					.child(div().child(EnumColumnToken))
					.child(
						ui::SelectField::new()
							.label(self.name.clone())
							.name(self.name.clone())
							.options(options)
							.value(self.value.to_string()),
					),
			)
			.child(Dehydrate::new(
				self.name,
				ColumnChart::Bar(
					BarChart::new()
						.hide_legend(true)
						.series(series)
						.should_draw_x_axis_labels(false)
						.should_draw_y_axis_labels(false),
				),
			))
			.into_node()
	}
}

pub struct TextField {
	pub name: String,
	pub value: FieldValue,
}

impl Component for TextField {
	fn into_node(self) -> Node {
		fragment()
			.child(
				div()
					.class("predict-field-wrapper")
					.child(div().child(TextColumnToken))
					.child(
						ui::TextField::new()
							.label(self.name.clone())
							.name(self.name)
							.value(self.value.to_string()),
					),
			)
			.child(div())
			.into_node()
	}
}
