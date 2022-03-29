pub use crate::{enum_column::*, number_column::*, text_column::*};
use pinwheel::prelude::*;
use modelfox_app_date_window::DateWindow;
use modelfox_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use modelfox_app_ui::date_window_select_field::DateWindowSelectField;
use modelfox_ui as ui;

pub struct Page {
	pub column_name: String,
	pub date_window: DateWindow,
	pub id: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Number(NumberColumn),
	Enum(EnumColumn),
	Text(TextColumn),
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

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Number(inner) => inner.into_node(),
			Inner::Enum(inner) => inner.into_node(),
			Inner::Text(inner) => inner.into_node(),
		};
		Document::new()
			.client("modelfox_app_production_stats_column_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new(self.column_name))
						.child(DateWindowSelectForm {
							date_window: self.date_window,
						})
						.child(inner),
				),
			)
			.into_node()
	}
}

pub struct DateWindowSelectForm {
	date_window: DateWindow,
}

impl Component for DateWindowSelectForm {
	fn into_node(self) -> Node {
		ui::Form::new()
			.child(DateWindowSelectField::new(self.date_window))
			.child(
				noscript().child(
					ui::Button::new()
						.button_type(ui::ButtonType::Submit)
						.child("Submit"),
				),
			)
			.into_node()
	}
}
