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
use html::{component, html, style, Props};
use num::ToPrimitive;
use tangram_finite::Finite;
use tangram_number_formatter::NumberFormatter;
use wasm_bindgen::JsCast;
use web_sys::*;

pub fn hydrate_chart<T>(id: &str)
where
	T: ChartImpl,
	T::Options: serde::de::DeserializeOwned,
{
	let window = window().unwrap();
	let document = window.document().unwrap();
	let container = document
		.get_element_by_id(id)
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let options = container.dataset().get("options").unwrap();
	let options = serde_json::from_str(&options).unwrap();
	let chart = Chart::<T>::new(container);
	chart.borrow_mut().draw(options);
	std::mem::forget(chart);
}

#[derive(Props)]
pub struct BarChartProps {
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

#[component]
pub fn BarChart(props: BarChartProps) {
	let options = BarChartOptions {
		group_gap: props.group_gap,
		hide_legend: props.hide_legend,
		number_formatter: NumberFormatter::default(),
		series: props.series.unwrap_or_else(Vec::new),
		should_draw_x_axis_labels: props.should_draw_x_axis_labels,
		should_draw_y_axis_labels: props.should_draw_y_axis_labels,
		x_axis_title: props.x_axis_title,
		y_axis_grid_line_interval: props.y_axis_grid_line_interval,
		y_axis_title: props.y_axis_title,
		y_max: props.y_max,
		y_min: props.y_min,
	};
	let hide_legend = props.hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<ChartLegendItemProps> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(ChartLegendItemProps {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={props.class}
				data-chart-type="bar"
				data-options={options}
				id={props.id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct BoxChartProps {
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

#[component]
pub fn BoxChart(props: BoxChartProps) {
	let options = BoxChartOptions {
		hide_legend: props.hide_legend,
		number_formatter: NumberFormatter::default(),
		series: props.series.unwrap_or_else(Vec::new),
		should_draw_x_axis_labels: props.should_draw_x_axis_labels,
		should_draw_y_axis_labels: props.should_draw_y_axis_labels,
		title: props.title.clone(),
		x_axis_title: props.x_axis_title,
		y_axis_title: props.y_axis_title,
		y_max: props.y_max,
		y_min: props.y_min,
	};
	let hide_legend = props.hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<ChartLegendItemProps> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(ChartLegendItemProps {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={props.class}
				data-chart-type="box"
				data-options={options}
				id={props.id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct FeatureContributionsChartProps {
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

#[component]
pub fn FeatureContributionsChart(props: FeatureContributionsChartProps) {
	let chart_config = ChartConfig::default();
	let n_series = props.series.len();
	let mut series = props.series;
	// Compress the feature contributions chart series on the server assuming a reasonable chart width to avoid sending too much data to the client.
	compress_feature_contributions_chart_series(
		series.as_mut_slice(),
		CompressFeatureContributionsChartSeriesOptions {
			chart_width: 2000.0,
			min_box_width: 8.0,
		},
	);
	let options = FeatureContributionsChartOptions {
		include_x_axis_title: props.include_x_axis_title,
		include_y_axis_labels: props.include_y_axis_labels,
		include_y_axis_title: props.include_y_axis_title,
		negative_color: props.negative_color,
		number_formatter: NumberFormatter::default(),
		positive_color: props.positive_color,
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
	let height =
		inner_chart_height
			+ top_padding
			+ label_padding
			+ font_size + if props.include_x_axis_title.unwrap_or(false) {
			label_padding + font_size
		} else {
			0.0
		} + label_padding
			+ font_size + bottom_padding;
	let container_style = style! {
		"height" => format!("{}px", height),
		"width" => "100%",
	};
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			<div
				class={props.class}
				data-chart-type="feature_contributions"
				data-options={options}
				id={props.id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct LineChartProps {
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

#[component]
pub fn LineChart(props: LineChartProps) {
	let options = LineChartOptions {
		hide_legend: props.hide_legend,
		labels: props.labels,
		number_formatter: NumberFormatter::default(),
		series: props.series.unwrap_or_else(Vec::new),
		should_draw_x_axis_labels: props.should_draw_x_axis_labels,
		should_draw_y_axis_labels: props.should_draw_y_axis_labels,
		title: props.title.clone(),
		x_axis_grid_line_interval: props.x_axis_grid_line_interval,
		x_axis_title: props.x_axis_title,
		x_max: props.x_max,
		x_min: props.x_min,
		y_axis_grid_line_interval: props.y_axis_grid_line_interval,
		y_axis_title: props.y_axis_title,
		y_max: props.y_max,
		y_min: props.y_min,
	};
	let hide_legend = props.hide_legend.unwrap_or(false);
	let container_style = style! {
		"padding-top" => "50%",
		"width" => "100%",
	};
	let legend_items: Vec<ChartLegendItemProps> = options
		.series
		.iter()
		.filter_map(|series| {
			let title = if let Some(title) = &series.title {
				title
			} else {
				return None;
			};
			Some(ChartLegendItemProps {
				color: series.color.clone(),
				title: title.clone(),
			})
		})
		.collect();
	let options = serde_json::to_string(&options).unwrap();
	html! {
		<div class="chart-wrapper">
			<ChartTitle>{props.title}</ChartTitle>
			{if !hide_legend {
				Some(html! { <ChartLegend items={legend_items} /> })
			} else {
				None
			}}
			<div
				class={props.class}
				data-chart-type="line"
				data-options={options}
				id={props.id}
				style={container_style}
			>
				<noscript>
					<div class="chart-noscript">
						{"Please enable JavaScript to view charts."}
					</div>
				</noscript>
			</div>
		</div>
	}
}

#[component]
pub fn ChartTitle() {
	html! {
		<div class="chart-title">{children}</div>
	}
}

#[derive(Props)]
pub struct ChartLegendProps {
	pub items: Vec<ChartLegendItemProps>,
}

#[component]
pub fn ChartLegend(props: ChartLegendProps) {
	html! {
		<div class="chart-legend-wrapper">
			{props.items.into_iter().map(|item| html! {
				<ChartLegendItem
					color={item.color}
					title={item.title}
				/>
			}).collect::<Vec<_>>()}
		</div>
	}
}

#[derive(Props)]
pub struct ChartLegendItemProps {
	pub color: String,
	pub title: String,
}

#[component]
fn ChartLegendItem(props: ChartLegendItemProps) {
	let style = style! {
		"background-color" => props.color,
	};
	html! {
		<div class="chart-legend-item">
			<div class="chart-legend-indicator" style={style}></div>
			<div class="chart-legend-title">{props.title}</div>
		</div>
	}
}
