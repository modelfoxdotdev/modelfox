use crate::{
	bar_chart::{BarChartOptions, BarChartSeries},
	box_chart::{BoxChartOptions, BoxChartSeries},
	chart::{Chart, ChartImpl},
	common::GridLineInterval,
	config::ChartConfig,
	feature_contributions_chart::CompressFeatureContributionsChartSeriesOptions,
	feature_contributions_chart::{
		compress_feature_contributions_chart_series, FeatureContributionsChartOptions,
		FeatureContributionsChartSeries,
	},
	line_chart::{LineChartOptions, LineChartSeries},
};
use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_finite::Finite;
use tangram_number_formatter::NumberFormatter;
use wasm_bindgen::JsCast;
use web_sys as dom;

pub fn hydrate_chart<T>(id: &str)
where
	T: ChartImpl,
	T::Options: serde::de::DeserializeOwned,
{
	let window = dom::window().unwrap();
	let document = window.document().unwrap();
	let container = document
		.get_element_by_id(id)
		.unwrap()
		.dyn_into::<dom::HtmlElement>()
		.unwrap();
	let options = container.dataset().get("options").unwrap();
	let options = serde_json::from_str(&options).unwrap();
	let chart = Chart::<T>::new(container);
	chart.borrow_mut().draw(options);
	std::mem::forget(chart);
}

#[derive(ComponentBuilder)]
pub struct BarChart {
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub group_gap: Option<f64>,
	#[optional]
	pub hide_legend: Option<bool>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub series: Option<Vec<BarChartSeries>>,
	#[optional]
	pub should_draw_x_axis_labels: Option<bool>,
	#[optional]
	pub should_draw_y_axis_labels: Option<bool>,
	#[optional]
	pub title: Option<String>,
	#[optional]
	pub x_axis_title: Option<String>,
	#[optional]
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	#[optional]
	pub y_axis_title: Option<String>,
	#[optional]
	pub y_max: Option<f64>,
	#[optional]
	pub y_min: Option<f64>,
}

impl Component for BarChart {
	fn into_node(self) -> Node {
		let options = BarChartOptions {
			group_gap: self.group_gap,
			hide_legend: self.hide_legend,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_else(Vec::new),
			should_draw_x_axis_labels: self.should_draw_x_axis_labels,
			should_draw_y_axis_labels: self.should_draw_y_axis_labels,
			x_axis_title: self.x_axis_title,
			y_axis_grid_line_interval: self.y_axis_grid_line_interval,
			y_axis_title: self.y_axis_title,
			y_max: self.y_max,
			y_min: self.y_min,
		};
		let hide_legend = self.hide_legend.unwrap_or(false);
		let legend_items: Vec<ChartLegendItem> = options
			.series
			.iter()
			.filter_map(|series| {
				let title = if let Some(title) = &series.title {
					title
				} else {
					return None;
				};
				Some(ChartLegendItem {
					color: series.color.clone(),
					title: title.clone(),
				})
			})
			.collect();
		let options = serde_json::to_string(&options).unwrap();
		div()
			.class("chart-wrapper")
			.child(ChartTitle::new().child(self.title))
			.child(if !hide_legend {
				Some(ChartLegend::new(legend_items))
			} else {
				None
			})
			.child(
				div()
					.attribute("class", self.class)
					.style(style::PADDING_TOP, "50%")
					.style(style::WIDTH, "100%")
					.attribute("data-chart-type", "bar")
					.attribute("data-options", options)
					.attribute("id", self.id)
					.child(
						noscript().child(
							div()
								.class("chart-noscript")
								.child("Please enable JavaScript to view charts."),
						),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct BoxChart {
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub hide_legend: Option<bool>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub series: Option<Vec<BoxChartSeries>>,
	#[optional]
	pub should_draw_x_axis_labels: Option<bool>,
	#[optional]
	pub should_draw_y_axis_labels: Option<bool>,
	#[optional]
	pub title: Option<String>,
	#[optional]
	pub x_axis_title: Option<String>,
	#[optional]
	pub y_axis_title: Option<String>,
	#[optional]
	pub y_max: Option<f64>,
	#[optional]
	pub y_min: Option<f64>,
}

impl Component for BoxChart {
	fn into_node(self) -> Node {
		let options = BoxChartOptions {
			hide_legend: self.hide_legend,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_else(Vec::new),
			should_draw_x_axis_labels: self.should_draw_x_axis_labels,
			should_draw_y_axis_labels: self.should_draw_y_axis_labels,
			title: self.title.clone(),
			x_axis_title: self.x_axis_title,
			y_axis_title: self.y_axis_title,
			y_max: self.y_max,
			y_min: self.y_min,
		};
		let hide_legend = self.hide_legend.unwrap_or(false);
		let legend_items: Vec<ChartLegendItem> = options
			.series
			.iter()
			.filter_map(|series| {
				let title = if let Some(title) = &series.title {
					title
				} else {
					return None;
				};
				Some(ChartLegendItem {
					color: series.color.clone(),
					title: title.clone(),
				})
			})
			.collect();
		let options = serde_json::to_string(&options).unwrap();
		div()
			.class("chart-wrapper")
			.child(ChartTitle::new().child(self.title))
			.child(if !hide_legend {
				Some(ChartLegend::new(legend_items))
			} else {
				None
			})
			.child(
				div()
					.attribute("class", self.class)
					.style(style::PADDING_TOP, "50%")
					.style(style::WIDTH, "100%")
					.attribute("data-chart-type", "box")
					.attribute("data-options", options)
					.attribute("id", self.id)
					.child(
						noscript().child(
							div()
								.class("chart-noscript")
								.child("Please enable JavaScript to view charts."),
						),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct FeatureContributionsChart {
	pub negative_color: String,
	pub positive_color: String,
	pub series: Vec<FeatureContributionsChartSeries>,
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub include_x_axis_title: Option<bool>,
	#[optional]
	pub include_y_axis_labels: Option<bool>,
	#[optional]
	pub include_y_axis_title: Option<bool>,
	#[optional]
	pub title: Option<String>,
}

impl Component for FeatureContributionsChart {
	fn into_node(self) -> Node {
		let chart_config = ChartConfig::default();
		let n_series = self.series.len();
		let mut series = self.series;
		// Compress the feature contributions chart series on the server assuming a reasonable chart width to avoid sending too much data to the client.
		compress_feature_contributions_chart_series(
			series.as_mut_slice(),
			CompressFeatureContributionsChartSeriesOptions {
				chart_width: 2000.0,
				min_box_width: 8.0,
			},
		);
		let options = FeatureContributionsChartOptions {
			include_x_axis_title: self.include_x_axis_title,
			include_y_axis_labels: self.include_y_axis_labels,
			include_y_axis_title: self.include_y_axis_title,
			negative_color: self.negative_color,
			number_formatter: NumberFormatter::default(),
			positive_color: self.positive_color,
			series,
		};
		let inner_chart_height = n_series.to_f64().unwrap()
			* chart_config.feature_contributions_series_height
			+ (n_series - 1).to_f64().unwrap() * chart_config.feature_contributions_series_gap;
		let ChartConfig {
			bottom_padding,
			font_size,
			label_padding,
			top_padding,
			..
		} = chart_config;
		let height = inner_chart_height
			+ top_padding
			+ label_padding
			+ font_size + if self.include_x_axis_title.unwrap_or(false) {
			label_padding + font_size
		} else {
			0.0
		} + label_padding
			+ font_size + bottom_padding;
		let options = serde_json::to_string(&options).unwrap();
		div()
			.class("chart-wrapper")
			.child(ChartTitle::new().child(self.title))
			.child(
				div()
					.attribute("class", self.class)
					.style(style::WIDTH, "100%")
					.style(style::HEIGHT, format!("{}px", height))
					.attribute("data-chart-type", "feature_contributions")
					.attribute("data-options", options)
					.attribute("id", self.id)
					.child(
						noscript().child(
							div()
								.class("chart-noscript")
								.child("Please enable JavaScript to view charts."),
						),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct LineChart {
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub hide_legend: Option<bool>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub labels: Option<Vec<String>>,
	#[optional]
	pub series: Option<Vec<LineChartSeries>>,
	#[optional]
	pub should_draw_x_axis_labels: Option<bool>,
	#[optional]
	pub should_draw_y_axis_labels: Option<bool>,
	#[optional]
	pub title: Option<String>,
	#[optional]
	pub x_axis_grid_line_interval: Option<GridLineInterval>,
	#[optional]
	pub x_axis_title: Option<String>,
	#[optional]
	pub x_max: Option<Finite<f64>>,
	#[optional]
	pub x_min: Option<Finite<f64>>,
	#[optional]
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	#[optional]
	pub y_axis_title: Option<String>,
	#[optional]
	pub y_max: Option<Finite<f64>>,
	#[optional]
	pub y_min: Option<Finite<f64>>,
}

impl Component for LineChart {
	fn into_node(self) -> Node {
		let options = LineChartOptions {
			hide_legend: self.hide_legend,
			labels: self.labels,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_else(Vec::new),
			should_draw_x_axis_labels: self.should_draw_x_axis_labels,
			should_draw_y_axis_labels: self.should_draw_y_axis_labels,
			title: self.title.clone(),
			x_axis_grid_line_interval: self.x_axis_grid_line_interval,
			x_axis_title: self.x_axis_title,
			x_max: self.x_max,
			x_min: self.x_min,
			y_axis_grid_line_interval: self.y_axis_grid_line_interval,
			y_axis_title: self.y_axis_title,
			y_max: self.y_max,
			y_min: self.y_min,
		};
		let hide_legend = self.hide_legend.unwrap_or(false);
		let legend_items: Vec<ChartLegendItem> = options
			.series
			.iter()
			.filter_map(|series| {
				let title = if let Some(title) = &series.title {
					title
				} else {
					return None;
				};
				Some(ChartLegendItem {
					color: series.color.clone(),
					title: title.clone(),
				})
			})
			.collect();
		let options = serde_json::to_string(&options).unwrap();
		div()
			.class("chart-wrapper")
			.child(ChartTitle::new().child(self.title))
			.child(if !hide_legend {
				Some(ChartLegend::new(legend_items))
			} else {
				None
			})
			.child(
				div()
					.attribute("class", self.class)
					.style(style::PADDING_TOP, "50%")
					.style(style::WIDTH, "100%")
					.attribute("data-chart-type", "line")
					.attribute("data-options", options)
					.attribute("id", self.id)
					.child(
						noscript().child(
							div()
								.class("chart-noscript")
								.child("Please enable JavaScript to view charts."),
						),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ChartTitle {
	#[children]
	pub children: Vec<Node>,
}
impl Component for ChartTitle {
	fn into_node(self) -> Node {
		div().class("chart-title").child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ChartLegend {
	pub items: Vec<ChartLegendItem>,
}

impl Component for ChartLegend {
	fn into_node(self) -> Node {
		div()
			.class("chart-legend-wrapper")
			.children(
				self.items
					.into_iter()
					.map(|item| ChartLegendItem::new(item.color, item.title)),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ChartLegendItem {
	pub color: String,
	pub title: String,
}

impl Component for ChartLegendItem {
	fn into_node(self) -> Node {
		div()
			.class("chart-legend-item")
			.child(
				div()
					.class("chart-legend-indicator")
					.style(style::BACKGROUND_COLOR, self.color),
			)
			.child(div().class("chart-legend-title").child(self.title))
			.into_node()
	}
}
