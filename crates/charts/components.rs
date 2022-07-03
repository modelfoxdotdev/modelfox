use crate::{
	bar_chart::{BarChartOptions, BarChartSeries},
	box_chart::{BoxChartOptions, BoxChartSeries},
	chart::Chart,
	common::GridLineInterval,
	config::ChartConfig,
	feature_contributions_chart::CompressFeatureContributionsChartSeriesOptions,
	feature_contributions_chart::{
		compress_feature_contributions_chart_series, FeatureContributionsChartOptions,
		FeatureContributionsChartSeries,
	},
	line_chart::{LineChartOptions, LineChartSeries},
};
use futures::future::pending;
use modelfox_finite::Finite;
use modelfox_number_formatter::NumberFormatter;
use num::ToPrimitive;
use pinwheel::prelude::*;
use std::borrow::Cow;
use wasm_bindgen::JsCast;

#[derive(builder, Default, new, serde::Serialize, serde::Deserialize)]
#[new(default)]
pub struct BarChart {
	#[builder]
	pub group_gap: Option<f64>,
	#[builder]
	pub hide_legend: Option<bool>,
	#[builder]
	pub series: Option<Vec<BarChartSeries>>,
	#[builder]
	pub should_draw_x_axis_labels: Option<bool>,
	#[builder]
	pub should_draw_y_axis_labels: Option<bool>,
	#[builder]
	pub title: Option<String>,
	#[builder]
	pub x_axis_title: Option<String>,
	#[builder]
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	#[builder]
	pub y_axis_title: Option<String>,
	#[builder]
	pub y_max: Option<f64>,
	#[builder]
	pub y_min: Option<f64>,
}

impl Component for BarChart {
	fn into_node(self) -> Node {
		let options = BarChartOptions {
			group_gap: self.group_gap,
			hide_legend: self.hide_legend,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_default(),
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
		let title = self.title.map(ChartTitle::new);
		let legend = if hide_legend {
			None
		} else {
			Some(ChartLegend {
				items: legend_items,
			})
		};
		let chart = div()
			.style(style::PADDING_TOP, "50%")
			.style(style::WIDTH, "100%")
			.future(|chart| {
				let chart = Chart::<crate::bar_chart::BarChart>::new(
					chart.dom_element().unchecked_into(),
					options,
				);
				async move {
					pending::<()>().await;
					drop(chart);
				}
			})
			.child(
				noscript().child(
					div()
						.class("chart-noscript")
						.child("Please enable JavaScript to view charts."),
				),
			);
		div()
			.class("chart-wrapper")
			.child(title)
			.child(legend)
			.child(chart)
			.into_node()
	}
}

#[derive(builder, Default, new, serde::Serialize, serde::Deserialize)]
#[new(default)]
pub struct BoxChart {
	#[builder]
	pub hide_legend: Option<bool>,
	#[builder]
	pub series: Option<Vec<BoxChartSeries>>,
	#[builder]
	pub should_draw_x_axis_labels: Option<bool>,
	#[builder]
	pub should_draw_y_axis_labels: Option<bool>,
	#[builder]
	pub title: Option<String>,
	#[builder]
	pub x_axis_title: Option<String>,
	#[builder]
	pub y_axis_title: Option<String>,
	#[builder]
	pub y_max: Option<f64>,
	#[builder]
	pub y_min: Option<f64>,
}

impl Component for BoxChart {
	fn into_node(self) -> Node {
		let options = BoxChartOptions {
			hide_legend: self.hide_legend,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_default(),
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
		let title = self.title.map(ChartTitle::new);
		let legend = if hide_legend {
			None
		} else {
			Some(ChartLegend {
				items: legend_items,
			})
		};
		let chart = div()
			.style(style::PADDING_TOP, "50%")
			.style(style::WIDTH, "100%")
			.future(|chart| {
				let chart = Chart::<crate::box_chart::BoxChart>::new(
					chart.dom_element().unchecked_into(),
					options,
				);
				async move {
					pending::<()>().await;
					drop(chart);
				}
			})
			.child(
				noscript().child(
					div()
						.class("chart-noscript")
						.child("Please enable JavaScript to view charts."),
				),
			);
		div()
			.class("chart-wrapper")
			.child(title)
			.child(legend)
			.child(chart)
			.into_node()
	}
}

#[derive(builder, Default, new, serde::Serialize, serde::Deserialize)]
#[new(default)]
pub struct FeatureContributionsChart {
	#[builder]
	pub series: Option<Vec<FeatureContributionsChartSeries>>,
	#[builder]
	pub negative_color: Option<String>,
	#[builder]
	pub positive_color: Option<String>,
	#[builder]
	pub include_x_axis_title: Option<bool>,
	#[builder]
	pub include_y_axis_labels: Option<bool>,
	#[builder]
	pub include_y_axis_title: Option<bool>,
	#[builder]
	pub title: Option<String>,
}

impl Component for FeatureContributionsChart {
	fn into_node(self) -> Node {
		let chart_config = ChartConfig::default();
		let mut series = self.series.unwrap_or_default();
		let n_series = series.len();
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
		let title = self.title.map(ChartTitle::new);
		let chart =
			div()
				.style(style::WIDTH, "100%")
				.style(style::HEIGHT, format!("{}px", height))
				.future(|chart| {
					let chart = Chart::<
						crate::feature_contributions_chart::FeatureContributionsChart,
					>::new(chart.dom_element().unchecked_into(), options);
					async move {
						pending::<()>().await;
						drop(chart);
					}
				})
				.child(
					noscript().child(
						div()
							.class("chart-noscript")
							.child("Please enable JavaScript to view charts."),
					),
				);
		div()
			.class("chart-wrapper")
			.child(title)
			.child(chart)
			.into_node()
	}
}

#[derive(builder, Default, new, serde::Serialize, serde::Deserialize)]
#[new(default)]
pub struct LineChart {
	#[builder]
	pub hide_legend: Option<bool>,
	#[builder]
	pub labels: Option<Vec<String>>,
	#[builder]
	pub series: Option<Vec<LineChartSeries>>,
	#[builder]
	pub should_draw_x_axis_labels: Option<bool>,
	#[builder]
	pub should_draw_y_axis_labels: Option<bool>,
	#[builder]
	pub title: Option<String>,
	#[builder]
	pub x_axis_grid_line_interval: Option<GridLineInterval>,
	#[builder]
	pub x_axis_title: Option<String>,
	#[builder]
	pub x_max: Option<Finite<f64>>,
	#[builder]
	pub x_min: Option<Finite<f64>>,
	#[builder]
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	#[builder]
	pub y_axis_title: Option<String>,
	#[builder]
	pub y_max: Option<Finite<f64>>,
	#[builder]
	pub y_min: Option<Finite<f64>>,
}

impl Component for LineChart {
	fn into_node(self) -> Node {
		let options = LineChartOptions {
			hide_legend: self.hide_legend,
			labels: self.labels,
			number_formatter: NumberFormatter::default(),
			series: self.series.unwrap_or_default(),
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
		let title = self.title.map(ChartTitle::new);
		let legend = if hide_legend {
			None
		} else {
			Some(ChartLegend {
				items: legend_items,
			})
		};
		let chart = div()
			.style(style::PADDING_TOP, "50%")
			.style(style::WIDTH, "100%")
			.future(|chart| {
				let chart = Chart::<crate::line_chart::LineChart>::new(
					chart.dom_element().unchecked_into(),
					options,
				);
				async move {
					pending::<()>().await;
					drop(chart);
				}
			})
			.child(
				noscript().child(
					div()
						.class("chart-noscript")
						.child("Please enable JavaScript to view charts."),
				),
			);
		div()
			.class("chart-wrapper")
			.child(title)
			.child(legend)
			.child(chart)
			.into_node()
	}
}

pub struct ChartTitle {
	pub title: Cow<'static, str>,
}

impl ChartTitle {
	pub fn new(title: impl Into<Cow<'static, str>>) -> ChartTitle {
		ChartTitle {
			title: title.into(),
		}
	}
}

impl Component for ChartTitle {
	fn into_node(self) -> Node {
		div().class("chart-title").child(self.title).into_node()
	}
}

pub struct ChartLegend {
	pub items: Vec<ChartLegendItem>,
}

impl Component for ChartLegend {
	fn into_node(self) -> Node {
		div()
			.class("chart-legend-wrapper")
			.children(self.items)
			.into_node()
	}
}

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
