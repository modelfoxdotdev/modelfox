use crate::{
	chart::{ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions, HoverRegion},
	common::{
		compute_rects, draw_rounded_rect, draw_x_axis, draw_x_axis_title, draw_y_axis_grid_lines,
		draw_y_axis_labels, draw_y_axis_title, ComputeRectsOptions, ComputeRectsOutput,
		DrawRoundedRectOptions, DrawXAxisOptions, DrawXAxisTitleOptions, DrawYAxisGridLinesOptions,
		DrawYAxisLabelsOptions, DrawYAxisTitleOptions, GridLineInterval, Point, Rect,
	},
	config::{ChartColors, ChartConfig},
	tooltip::{draw_tooltip, DrawTooltipOptions, TooltipLabel},
};
use itertools::Itertools;
use num::ToPrimitive;
use tangram_number_formatter::NumberFormatter;
use web_sys::*;

pub struct BarChart;

impl ChartImpl for BarChart {
	type Options = BarChartOptions;
	type OverlayInfo = BarChartOverlayInfo;
	type HoverRegionInfo = BarChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_bar_chart(options)
	}

	fn draw_overlay(
		options: DrawOverlayOptions<Self::Options, Self::OverlayInfo, Self::HoverRegionInfo>,
	) {
		draw_bar_chart_overlay(options)
	}
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BarChartOptions {
	pub group_gap: Option<f64>,
	pub hide_legend: Option<bool>,
	pub number_formatter: NumberFormatter,
	pub series: Vec<BarChartSeries>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub x_axis_title: Option<String>,
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<f64>,
	pub y_min: Option<f64>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BarChartSeries {
	pub color: String,
	pub data: Vec<BarChartPoint>,
	pub title: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BarChartPoint {
	pub label: String,
	pub x: f64,
	pub y: Option<f64>,
}

pub struct BarChartOverlayInfo {
	pub chart_rect: Rect,
}

#[derive(Clone)]
pub struct BarChartHoverRegionInfo {
	pub rect: Rect,
	pub color: String,
	pub point: BarChartPoint,
	pub point_label: String,
	pub point_value: f64,
	pub series_title: Option<String>,
	pub tooltip_origin_pixels: Point,
}

fn draw_bar_chart(
	options: DrawChartOptions<BarChartOptions>,
) -> DrawChartOutput<BarChartOverlayInfo, BarChartHoverRegionInfo> {
	let DrawChartOptions {
		chart_colors,
		chart_config,
		ctx,
		options,
		..
	} = options;
	let BarChartOptions {
		series,
		x_axis_title,
		y_axis_grid_line_interval,
		y_axis_title,
		..
	} = options;
	let width = ctx.canvas().unwrap().client_width().to_f64().unwrap();
	let height = ctx.canvas().unwrap().client_height().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<BarChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let y_min: Option<f64> = options.y_min.or_else(|| {
		series
			.iter()
			.flat_map(|series| series.data.iter().map(|p| p.y))
			.flatten()
			.min_by(|a, b| a.partial_cmp(b).unwrap())
	});
	let y_max = options.y_max.or_else(|| {
		series
			.iter()
			.flat_map(|series| series.data.iter().map(|p| p.y))
			.flatten()
			.max_by(|a, b| a.partial_cmp(b).unwrap())
	});
	let (y_min, y_max) = match (y_min, y_max) {
		(Some(y_min), Some(y_max)) => {
			if (y_max - y_min).abs() < f64::EPSILON {
				(y_min, y_min + 1.0)
			} else {
				(y_min, y_max)
			}
		}
		_ => (0.0, 1.0),
	};

	// Compute the boxes.
	let ComputeRectsOutput {
		chart_rect,
		x_axis_labels_rect,
		x_axis_title_rect,
		y_axis_grid_line_info,
		y_axis_labels_rect,
		y_axis_title_rect,
	} = compute_rects(ComputeRectsOptions {
		chart_config,
		ctx,
		height,
		include_x_axis_labels: options.should_draw_x_axis_labels.unwrap_or(true),
		include_x_axis_title: x_axis_title.is_some(),
		include_y_axis_labels: options.should_draw_y_axis_labels.unwrap_or(true),
		include_y_axis_title: y_axis_title.is_some(),
		number_formatter: &options.number_formatter,
		width,
		x_axis_grid_line_interval: None,
		y_axis_grid_line_interval: y_axis_grid_line_interval.as_ref(),
		y_max,
		y_min,
	});

	let categories: Vec<_> = series
		.get(0)
		.unwrap()
		.data
		.iter()
		.map(|p| &p.label)
		.collect();
	let n_categories = categories.len().to_f64().unwrap();
	let n_series = series.len().to_f64().unwrap();
	let group_gap = options.group_gap.unwrap_or(chart_config.bar_group_gap);
	let bar_group_width = (chart_rect.w - group_gap * (n_categories + 1.0)) / n_categories;
	let bar_width = (bar_group_width - chart_config.bar_gap * (n_series - 1.0)) / n_series;

	// Draw the X axis labels.
	if options.should_draw_x_axis_labels.unwrap_or(true) {
		draw_bar_chart_x_axis_labels(DrawBarChartXAxisLabelsOptions {
			bar_group_gap: group_gap,
			chart_colors,
			rect: x_axis_labels_rect,
			categories: &categories,
			ctx,
			group_width: bar_group_width,
			width,
		})
	}

	draw_y_axis_grid_lines(DrawYAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		y_axis_grid_line_info: &y_axis_grid_line_info,
	});

	draw_x_axis(DrawXAxisOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		y_axis_grid_line_info: &y_axis_grid_line_info,
	});

	// Draw the Y axis labels.
	if options.should_draw_y_axis_labels.unwrap_or(true) {
		draw_y_axis_labels(DrawYAxisLabelsOptions {
			chart_colors,
			rect: y_axis_labels_rect,
			ctx,
			font_size: chart_config.font_size,
			grid_line_info: &y_axis_grid_line_info,
			height,
			number_formatter: &options.number_formatter,
		})
	}

	// Draw the X axis title.
	if let Some(x_axis_title) = x_axis_title {
		draw_x_axis_title(DrawXAxisTitleOptions {
			chart_colors,
			rect: x_axis_title_rect,
			ctx,
			title: x_axis_title,
		});
	}

	// Draw the Y axis title.
	if let Some(y_axis_title) = y_axis_title {
		draw_y_axis_title(DrawYAxisTitleOptions {
			chart_colors,
			rect: y_axis_title_rect,
			ctx,
			title: y_axis_title,
		});
	}

	// Draw the bars.
	let series_len = series.len();
	let has_multiple_series = series_len > 1;
	for (series_index, series) in series.iter().enumerate() {
		for (point_index, point) in series.data.iter().enumerate() {
			if let Some(y) = point.y {
				let h = if y < 0.0 {
					(y - y_max.min(0.0)) / (y_max - y_min) * chart_rect.h
				} else {
					(y - y_min.max(0.0)) / (y_max - y_min) * chart_rect.h
				};
				let rect = Rect {
					h,
					w: (bar_group_width
						- chart_config.bar_gap * (series_len - 1).to_f64().unwrap())
						/ series_len.to_f64().unwrap(),
					x: chart_rect.x
						+ (group_gap
							+ (group_gap + bar_group_width) * point_index.to_f64().unwrap())
						+ (chart_config.bar_gap + bar_width) * series_index.to_f64().unwrap(),
					y: chart_rect.y + ((y_max - y) / (y_max - y_min)) * chart_rect.h,
				};
				draw_bar(DrawBarOptions {
					rect,
					chart_config,
					color: &format!("{}af", series.color),
					ctx,
				});
				let hover_region = HoverRegion {
					distance: Box::new(move |x, _| (rect.x + rect.w / 2.0 - x).abs()),
					hit_test: Box::new(move |x, y| {
						x >= rect.x
							&& x < rect.x + rect.w && y >= chart_rect.y
							&& y < chart_rect.y + chart_rect.h
					}),
					info: BarChartHoverRegionInfo {
						rect,
						color: series.color.clone(),
						point: point.clone(),
						point_label: point.label.clone(),
						point_value: y,
						series_title: if has_multiple_series {
							series.title.clone()
						} else {
							None
						},
						tooltip_origin_pixels: Point {
							x: rect.x + rect.w / 2.0,
							y: rect.y,
						},
					},
				};
				hover_regions.push(hover_region);
			}
		}
	}

	let overlay_info = BarChartOverlayInfo { chart_rect };

	DrawChartOutput {
		hover_regions,
		overlay_info,
	}
}

struct DrawBarOptions<'a> {
	chart_config: &'a ChartConfig,
	color: &'a str,
	ctx: &'a CanvasRenderingContext2d,
	rect: Rect,
}

fn draw_bar(options: DrawBarOptions) {
	let DrawBarOptions {
		chart_config,
		color,
		ctx,
		rect,
	} = options;
	let (round_bottom_left, round_bottom_right, round_top_left, round_top_right) = if rect.h > 0.0 {
		(false, false, true, true)
	} else {
		(true, true, false, false)
	};
	let radius = f64::INFINITY
		.min((rect.h / 2.0).abs())
		.min((rect.w / 6.0).abs())
		.min(chart_config.max_corner_radius);
	draw_rounded_rect(DrawRoundedRectOptions {
		rect,
		round_bottom_left,
		round_bottom_right,
		round_top_left,
		round_top_right,
		ctx,
		fill_color: Some(color),
		radius,
		stroke_color: Some(color),
		stroke_width: Some(chart_config.bar_stroke_width),
	})
}

pub struct DrawBarChartXAxisLabelsOptions<'a> {
	pub bar_group_gap: f64,
	pub chart_colors: &'a ChartColors,
	pub rect: Rect,
	pub categories: &'a [&'a String],
	pub ctx: &'a CanvasRenderingContext2d,
	pub group_width: f64,
	pub width: f64,
}

pub fn draw_bar_chart_x_axis_labels(options: DrawBarChartXAxisLabelsOptions) {
	let DrawBarChartXAxisLabelsOptions {
		bar_group_gap,
		chart_colors,
		rect,
		categories,
		ctx,
		group_width,
		width,
	} = options;
	ctx.save();
	ctx.set_fill_style(&chart_colors.label_color.into());
	ctx.set_text_baseline("bottom");
	ctx.set_text_align("center");
	// Find the smallest label step size at which labels do not overlap.
	let label_widths: Vec<f64> = categories
		.iter()
		.map(|label| ctx.measure_text(label).unwrap().width())
		.collect();
	let mut label_step_size = 1;
	loop {
		// This is how far the next label's center is.
		let next_label_offset = (bar_group_gap + group_width) * label_step_size.to_f64().unwrap();
		let overlap = label_widths
			.iter()
			.step_by(label_step_size)
			.tuple_windows()
			.any(|(label_width, next_label_width)| {
				label_width / 2.0 + next_label_width / 2.0 > next_label_offset
			});
		if overlap {
			label_step_size += 1;
			continue;
		} else {
			break;
		}
	}
	// Render every `label_step_size` label.
	for (label_index, label) in categories.iter().enumerate().step_by(label_step_size) {
		let label_offset = bar_group_gap
			+ group_width / 2.0
			+ (bar_group_gap + group_width) * label_index.to_f64().unwrap();
		// Do not draw the label if it will overflow the chart.
		let overflow_left =
			rect.x + label_offset - ctx.measure_text(label).unwrap().width() / 2.0 < 0.0;
		let overflow_right =
			rect.x + label_offset + ctx.measure_text(label).unwrap().width() / 2.0 > width;
		if overflow_left || overflow_right {
			continue;
		}
		ctx.fill_text(label, rect.x + label_offset, rect.y + rect.h)
			.unwrap();
	}
	ctx.restore();
}

fn draw_bar_chart_overlay(
	options: DrawOverlayOptions<BarChartOptions, BarChartOverlayInfo, BarChartHoverRegionInfo>,
) {
	if let Some(active_hover_region) = options.active_hover_regions.get(0) {
		let series_title = &active_hover_region.info.series_title;
		let point_label = &active_hover_region.info.point_label;
		let point_value = options
			.options
			.number_formatter
			.format(active_hover_region.info.point_value);
		let text = if let Some(series_title) = series_title {
			format!("{} ({}, {})", series_title, point_label, point_value)
		} else {
			format!("({}, {})", point_label, point_value)
		};
		let tooltip_label = TooltipLabel {
			color: active_hover_region.info.color.clone(),
			text,
		};
		draw_tooltip(DrawTooltipOptions {
			center_horizontal: Some(true),
			chart_colors: &options.chart_colors,
			chart_config: &options.chart_config,
			container: options.overlay_div,
			flip_y_offset: None,
			labels: vec![tooltip_label],
			origin: active_hover_region.info.tooltip_origin_pixels,
		});
		draw_bar(DrawBarOptions {
			chart_config: &options.chart_config,
			color: "#00000022",
			ctx: options.ctx,
			rect: active_hover_region.info.rect,
		})
	}
}
