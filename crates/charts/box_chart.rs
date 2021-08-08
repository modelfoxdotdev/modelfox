use crate::{
	bar_chart::{draw_bar_chart_x_axis_labels, DrawBarChartXAxisLabelsOptions},
	chart::{ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions, HoverRegion},
	common::{
		compute_rects, draw_rounded_rect, draw_x_axis, draw_x_axis_title, draw_y_axis_grid_lines,
		draw_y_axis_labels, draw_y_axis_title, ComputeRectsOptions, ComputeRectsOutput,
		DrawRoundedRectOptions, DrawXAxisOptions, DrawXAxisTitleOptions, DrawYAxisGridLinesOptions,
		DrawYAxisLabelsOptions, DrawYAxisTitleOptions, Point, Rect,
	},
	config::ChartConfig,
	tooltip::{draw_tooltip, DrawTooltipOptions, TooltipLabel},
};
use num::ToPrimitive;
use tangram_number_formatter::NumberFormatter;
use wasm_bindgen::JsValue;
use web_sys as dom;

pub struct BoxChart;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartOptions {
	pub hide_legend: Option<bool>,
	pub number_formatter: NumberFormatter,
	pub series: Vec<BoxChartSeries>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub title: Option<String>,
	pub x_axis_title: Option<String>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<f64>,
	pub y_min: Option<f64>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartSeries {
	pub color: String,
	pub data: Vec<BoxChartPoint>,
	pub title: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartPoint {
	pub label: String,
	pub x: f64,
	pub y: Option<BoxChartValue>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BoxChartValue {
	pub max: f64,
	pub min: f64,
	pub p25: f64,
	pub p50: f64,
	pub p75: f64,
}

pub struct BoxChartOverlayInfo {
	chart_rect: Rect,
}

#[derive(Clone)]
pub struct BoxChartHoverRegionInfo {
	color: String,
	label: String,
	name: String,
	tooltip_origin_pixels: Point,
	value: f64,
}

impl ChartImpl for BoxChart {
	type Options = BoxChartOptions;
	type OverlayInfo = BoxChartOverlayInfo;
	type HoverRegionInfo = BoxChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_box_chart(options)
	}

	fn draw_overlay(
		options: DrawOverlayOptions<Self::Options, Self::OverlayInfo, Self::HoverRegionInfo>,
	) {
		draw_box_chart_overlay(options)
	}
}

fn draw_box_chart(
	options: DrawChartOptions<BoxChartOptions>,
) -> DrawChartOutput<BoxChartOverlayInfo, BoxChartHoverRegionInfo> {
	let DrawChartOptions {
		chart_colors,
		chart_config,
		ctx,
		options,
	} = options;
	let BoxChartOptions {
		series,
		x_axis_title,
		y_axis_title,
		..
	} = &options;
	let canvas = ctx.canvas().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let y_min: Option<f64> = options.y_min.or_else(|| {
		series
			.iter()
			.flat_map(|series| series.data.iter().map(|p| p.y.as_ref().map(|y| y.min)))
			.flatten()
			.min_by(|a, b| a.partial_cmp(b).unwrap())
	});
	let y_max = options.y_max.or_else(|| {
		series
			.iter()
			.flat_map(|series| series.data.iter().map(|p| p.y.as_ref().map(|y| y.max)))
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
		y_axis_grid_line_interval: None,
		y_max,
		y_min,
	});

	let categories: Vec<&String> = series[0].data.iter().map(|point| &point.label).collect();
	let box_group_width = (chart_rect.w
		- chart_config.bar_group_gap * (categories.len() + 1).to_f64().unwrap())
		/ categories.len().to_f64().unwrap();

	// Draw the X axis labels.
	if options.should_draw_x_axis_labels.unwrap_or(true) {
		draw_bar_chart_x_axis_labels(DrawBarChartXAxisLabelsOptions {
			bar_group_gap: chart_config.bar_group_gap,
			chart_colors,
			rect: x_axis_labels_rect,
			categories: &categories,
			ctx,
			group_width: box_group_width,
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

	// Draw the boxes.
	for (series_index, this_series) in series.iter().enumerate() {
		for (point_index, point) in this_series.data.iter().enumerate() {
			let output = draw_box(DrawBoxOptions {
				box_gap: chart_config.bar_gap,
				box_group_gap: chart_config.bar_group_gap,
				box_group_width,
				chart_rect,
				chart_config,
				point_index,
				series_index,
				y_max,
				y_min,
				ctx,
				data: series,
				point,
				series: this_series,
			});
			hover_regions.extend(output.hover_regions);
		}
	}

	let overlay_info = BoxChartOverlayInfo { chart_rect };

	DrawChartOutput {
		hover_regions,
		overlay_info,
	}
}

fn draw_box_chart_overlay(
	options: DrawOverlayOptions<BoxChartOptions, BoxChartOverlayInfo, BoxChartHoverRegionInfo>,
) {
	let DrawOverlayOptions {
		active_hover_regions,
		ctx,
		options,
		overlay_info,
		overlay_div,
		chart_config,
		chart_colors,
	} = options;
	let BoxChartOverlayInfo { chart_rect } = overlay_info;
	let mut tooltips: Vec<TooltipLabel> = Vec::new();
	fn box_point_index_for_name(name: &str) -> usize {
		match name {
			"max" => 4,
			"median" => 2,
			"min" => 0,
			"p25" => 1,
			"p75" => 3,
			_ => unreachable!(),
		}
	}
	let mut active_hover_regions = active_hover_regions.to_owned();
	active_hover_regions.sort_unstable_by_key(|r| box_point_index_for_name(&r.info.name));
	for active_hover_region in active_hover_regions.iter() {
		let color = &active_hover_region.info.color;
		let x = &active_hover_region.info.label;
		let name = &active_hover_region.info.name;
		let value = options
			.number_formatter
			.format(active_hover_region.info.value);
		let y = format!("{} = {}", name, value);
		let text = format!("({}, {})", x, y);
		tooltips.push(TooltipLabel {
			color: color.to_owned(),
			text,
		});
		draw_line(DrawLineOptions {
			color: Some(chart_colors.crosshairs_color),
			ctx,
			dashed: Some(true),
			end: Point {
				x: chart_rect.x + chart_rect.w,
				y: active_hover_region.info.tooltip_origin_pixels.y,
			},
			start: Point {
				x: chart_rect.x,
				y: active_hover_region.info.tooltip_origin_pixels.y,
			},
			line_cap: None,
			line_width: None,
		});
	}
	if !tooltips.is_empty() {
		let last_active_hover_region = active_hover_regions.last().unwrap();
		let origin = last_active_hover_region.info.tooltip_origin_pixels;
		draw_tooltip(DrawTooltipOptions {
			center_horizontal: Some(true),
			container: overlay_div,
			labels: tooltips,
			origin,
			chart_colors,
			chart_config,
			flip_y_offset: None,
		})
	}
}

struct DrawBoxOptions<'a> {
	box_gap: f64,
	box_group_gap: f64,
	box_group_width: f64,
	chart_rect: Rect,
	chart_config: &'a ChartConfig,
	ctx: &'a dom::CanvasRenderingContext2d,
	data: &'a [BoxChartSeries],
	point: &'a BoxChartPoint,
	point_index: usize,
	series: &'a BoxChartSeries,
	series_index: usize,
	y_max: f64,
	y_min: f64,
}

struct DrawBoxOutput {
	hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>>,
}

fn draw_box(options: DrawBoxOptions) -> DrawBoxOutput {
	let DrawBoxOptions {
		chart_config,
		box_gap,
		box_group_gap,
		box_group_width,
		chart_rect,
		ctx,
		data,
		point,
		point_index,
		series,
		series_index,
		y_max,
		y_min,
	} = options;
	let n_series = data.len().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<BoxChartHoverRegionInfo>> = Vec::new();

	let value = if let Some(y) = point.y.as_ref() {
		y
	} else {
		return DrawBoxOutput { hover_regions };
	};

	let box_width = box_group_width / n_series - chart_config.bar_gap * (n_series - 1.0);
	let x = chart_rect.x
		+ (box_group_gap + (box_group_gap + box_group_width) * point_index.to_f64().unwrap())
		+ (box_gap + box_width) * series_index.to_f64().unwrap();

	let whisker_tip_width = box_width / 10.0;
	let line_width = 2.0;
	let value_to_pixels = |value: f64| {
		chart_rect.y + chart_rect.h
			- (-y_min / (y_max - y_min)) * chart_rect.h
			- (value / (y_max - y_min)) * chart_rect.h
	};

	// Draw the box.
	let rect = Rect {
		h: ((value.p75 - value.p25).abs() / (y_max - y_min)) * chart_rect.h,
		w: box_width,
		x,
		y: value_to_pixels(f64::max(value.p25, value.p75)),
	};
	let radius = f64::INFINITY
		.min((rect.h / 2.0).abs())
		.min((rect.w / 6.0).abs())
		.min(chart_config.max_corner_radius);
	draw_rounded_rect(DrawRoundedRectOptions {
		rect,
		ctx,
		fill_color: Some(&format!("{}af", series.color)),
		radius,
		stroke_color: Some(&series.color),
		stroke_width: Some(chart_config.bar_stroke_width),
		round_bottom_left: true,
		round_bottom_right: true,
		round_top_left: true,
		round_top_right: true,
	});

	// Create a clip path so the median line will not overflow the box.
	ctx.save();
	draw_rounded_rect(DrawRoundedRectOptions {
		ctx,
		fill_color: None,
		radius,
		rect,
		round_bottom_left: true,
		round_bottom_right: true,
		round_top_left: true,
		round_top_right: true,
		stroke_color: None,
		stroke_width: None,
	});
	ctx.clip();
	// Draw the median line.
	let median_rect = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p50),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: median_rect.x + median_rect.w,
			y: median_rect.y,
		},
		line_width: Some(line_width),
		start: Point {
			x: median_rect.x,
			y: median_rect.y,
		},
		dashed: None,
		line_cap: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: median_rect,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "median".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: median_rect.y,
		},
		value: value.p50,
		chart_config,
	}));
	ctx.restore();

	// Draw the min line.
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.min),
		},
		line_width: Some(line_width),
		start: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.p25),
		},
		dashed: None,
		line_cap: None,
	});
	let min_whisker_tip_rect = Rect {
		h: line_width,
		w: whisker_tip_width,
		x: x + box_width / 2.0 - whisker_tip_width / 2.0,
		y: value_to_pixels(value.min),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: min_whisker_tip_rect.x + min_whisker_tip_rect.w,
			y: min_whisker_tip_rect.y,
		},
		line_cap: Some("round"),
		line_width: Some(line_width),
		start: Point {
			x: min_whisker_tip_rect.x,
			y: min_whisker_tip_rect.y,
		},
		dashed: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: min_whisker_tip_rect,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "min".to_owned(),
		tooltip_origin_pixels: Point {
			y: min_whisker_tip_rect.y,
			x: x + box_width / 2.0,
		},
		value: value.min,
		chart_config,
	}));

	// Draw the max line.
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.max),
		},
		line_width: Some(line_width),
		start: Point {
			x: x + box_width / 2.0,
			y: value_to_pixels(value.p75),
		},
		dashed: None,
		line_cap: None,
	});
	let max_whisker_tip_rect = Rect {
		h: line_width,
		w: whisker_tip_width,
		x: x + box_width / 2.0 - whisker_tip_width / 2.0,
		y: value_to_pixels(value.max),
	};
	draw_line(DrawLineOptions {
		color: Some(&series.color),
		ctx,
		end: Point {
			x: max_whisker_tip_rect.x + max_whisker_tip_rect.w,
			y: max_whisker_tip_rect.y,
		},
		line_cap: Some("round"),
		line_width: Some(line_width),
		start: Point {
			x: max_whisker_tip_rect.x,
			y: max_whisker_tip_rect.y,
		},
		dashed: None,
	});
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: max_whisker_tip_rect,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "max".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: max_whisker_tip_rect.y,
		},
		value: value.max,
		chart_config,
	}));

	// Register the p25 hit region.
	let p25_rect = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p25),
	};
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: p25_rect,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "p25".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: p25_rect.y,
		},
		value: value.p25,
		chart_config,
	}));

	// Register the p75 hit region.
	let p75_rect = Rect {
		h: line_width,
		w: box_width,
		x,
		y: value_to_pixels(value.p75),
	};
	hover_regions.push(box_chart_hover_region(BoxChartHoverRegionOptions {
		rect: p75_rect,
		color: series.color.clone(),
		label: point.label.clone(),
		name: "p75".to_owned(),
		tooltip_origin_pixels: Point {
			x: x + box_width / 2.0,
			y: p75_rect.y,
		},
		value: value.p75,
		chart_config,
	}));

	DrawBoxOutput { hover_regions }
}

struct BoxChartHoverRegionOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	color: String,
	label: String,
	name: String,
	tooltip_origin_pixels: Point,
	value: f64,
}

fn box_chart_hover_region(
	options: BoxChartHoverRegionOptions,
) -> HoverRegion<BoxChartHoverRegionInfo> {
	let BoxChartHoverRegionOptions {
		chart_config,
		rect,
		color,
		label,
		name,
		tooltip_origin_pixels,
		value,
	} = options;
	let tooltip_target_radius = chart_config.tooltip_target_radius;
	HoverRegion {
		distance: Box::new(move |x, y| (rect.x - x).powi(2) + (rect.y - y).powi(2)),
		hit_test: Box::new(move |x, y| {
			y < rect.y + rect.h + tooltip_target_radius
				&& y > rect.y - rect.h - tooltip_target_radius
				&& x > rect.x - tooltip_target_radius
				&& x < rect.x + rect.w + tooltip_target_radius
		}),
		info: BoxChartHoverRegionInfo {
			color,
			label,
			name,
			tooltip_origin_pixels,
			value,
		},
	}
}

struct DrawLineOptions<'a> {
	color: Option<&'a str>,
	ctx: &'a dom::CanvasRenderingContext2d,
	dashed: Option<bool>,
	end: Point,
	line_cap: Option<&'a str>,
	line_width: Option<f64>,
	start: Point,
}

fn draw_line(options: DrawLineOptions) {
	let DrawLineOptions {
		color,
		ctx,
		dashed,
		end,
		line_cap,
		line_width,
		start,
	} = options;
	let line_width = line_width.unwrap_or(1.0);
	let dashed = dashed.unwrap_or(false);
	let line_cap = line_cap.as_deref().unwrap_or("butt");
	ctx.save();
	if dashed {
		ctx.set_line_dash(&JsValue::from_serde(&[4.0, 4.0]).unwrap())
			.unwrap();
	}
	if let Some(color) = &color {
		ctx.set_stroke_style(&color.to_owned().into());
	}
	ctx.set_line_width(line_width);
	ctx.set_line_cap(line_cap);
	ctx.begin_path();
	ctx.move_to(start.x, start.y);
	ctx.line_to(end.x, end.y);
	ctx.stroke();
	ctx.restore();
}
