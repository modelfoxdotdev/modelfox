use super::config::{ChartColors, ChartConfig};
use modelfox_number_formatter::NumberFormatter;
use num::ToPrimitive;
use std::borrow::Cow;
use web_sys as dom;

// |--------------------------------------------------|
// |  | |                                             |
// |  | |                                             |
// |  | |                                             |
// |YT|Y|                 Chart Box                   |
// |  | |                                             |
// |  | |                                             |
// |  | |                                             |
// |--------------------------------------------------|
// |   	|                X Axis Labels                |
// |    ----------------------------------------------|
// |   	|                X Axis Title                 |
// |--------------------------------------------------|

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy)]
pub struct ComputeRectsOptions<'a> {
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub height: f64,
	pub include_x_axis_labels: bool,
	pub include_x_axis_title: bool,
	pub include_y_axis_labels: bool,
	pub include_y_axis_title: bool,
	pub number_formatter: &'a NumberFormatter,
	pub width: f64,
	pub x_axis_grid_line_interval: Option<&'a GridLineInterval>,
	pub y_axis_grid_line_interval: Option<&'a GridLineInterval>,
	pub y_max: f64,
	pub y_min: f64,
}

pub struct ComputeRectsOutput {
	pub chart_rect: Rect,
	pub x_axis_labels_rect: Rect,
	pub x_axis_title_rect: Rect,
	pub y_axis_grid_line_info: GridLineInfo,
	pub y_axis_labels_rect: Rect,
	pub y_axis_title_rect: Rect,
}

#[derive(Clone, Copy)]
pub struct Point {
	pub x: f64,
	pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Rect {
	pub x: f64,
	pub y: f64,
	pub w: f64,
	pub h: f64,
}

/// The interval is k * 10 ** p. k will always be 1, 2, or 5.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GridLineInterval {
	pub k: f64,
	pub p: f64,
}

#[derive(Clone)]
pub struct GridLineInfo {
	pub interval: f64,
	pub interval_pixels: f64,
	pub k: f64,
	pub num_grid_lines: usize,
	pub p: f64,
	pub start: f64,
	pub start_pixels: f64,
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn compute_rects(options: ComputeRectsOptions) -> ComputeRectsOutput {
	let ComputeRectsOptions {
		chart_config:
			&ChartConfig {
				bottom_padding,
				font_size,
				label_padding,
				left_padding,
				right_padding,
				top_padding,
				..
			},
		ctx,
		height,
		include_x_axis_labels,
		include_x_axis_title,
		include_y_axis_labels,
		include_y_axis_title,
		number_formatter,
		width,
		y_axis_grid_line_interval,
		y_max,
		y_min,
		..
	} = options;

	let chart_height = height
		- top_padding
		- if include_x_axis_labels {
			label_padding + font_size
		} else {
			0.0
		} - if include_x_axis_title {
		label_padding + font_size
	} else {
		0.0
	} - bottom_padding;

	let y_axis_grid_line_info = compute_y_axis_grid_line_info(ComputeYAxisGridLineInfoOptions {
		chart_height,
		font_size,
		y_axis_grid_line_interval,
		y_max,
		y_min,
	});

	let y_axis_labels_width =
		compute_axis_labels_max_width(ctx, &y_axis_grid_line_info, number_formatter);

	let chart_width = width
		- left_padding
		- if include_y_axis_title {
			font_size + label_padding
		} else {
			0.0
		} - if include_y_axis_labels {
		y_axis_labels_width + label_padding
	} else {
		0.0
	} - right_padding;

	let x_axis_labels_height = if include_x_axis_labels {
		font_size
	} else {
		0.0
	};

	let x_axis_title_height = if include_x_axis_title { font_size } else { 0.0 };

	let x_axis_labels_rect = Rect {
		h: x_axis_labels_height,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				font_size + label_padding
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		},
		y: top_padding
			+ chart_height
			+ if include_x_axis_labels {
				label_padding
			} else {
				0.0
			},
	};

	let x_axis_title_rect = Rect {
		h: if include_x_axis_title {
			x_axis_title_height
		} else {
			0.0
		},
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				font_size + label_padding
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		},
		y: top_padding
			+ chart_height
			+ if include_x_axis_labels {
				label_padding + font_size
			} else {
				0.0
			} + if include_x_axis_title {
			label_padding
		} else {
			0.0
		},
	};

	let y_axis_title_rect = Rect {
		h: chart_height,
		w: font_size,
		x: left_padding,
		y: top_padding,
	};

	let y_axis_labels_rect = Rect {
		h: chart_height,
		w: y_axis_labels_width,
		x: left_padding
			+ if include_x_axis_title {
				font_size + label_padding
			} else {
				0.0
			},
		y: top_padding,
	};

	let chart_rect = Rect {
		h: chart_height,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				font_size + label_padding
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		},
		y: top_padding,
	};

	ComputeRectsOutput {
		chart_rect,
		x_axis_labels_rect,
		x_axis_title_rect,
		y_axis_grid_line_info,
		y_axis_labels_rect,
		y_axis_title_rect,
	}
}

fn compute_grid_line_interval(
	min: f64,
	max: f64,
	distance_pixels: f64,
	min_grid_line_distance_pixels: f64,
) -> GridLineInterval {
	let range = max - min;
	let ideal_n = (distance_pixels / min_grid_line_distance_pixels).floor();
	let ideal_interval = range / ideal_n;
	let ideal_p = ideal_interval.log10().floor();
	let ideal_k = ideal_interval / 10.0f64.powf(ideal_p);
	if ideal_k <= 2.0 {
		GridLineInterval { k: 2.0, p: ideal_p }
	} else if ideal_k <= 5.0 {
		GridLineInterval { k: 5.0, p: ideal_p }
	} else {
		GridLineInterval {
			k: 1.0,
			p: ideal_p + 1.0,
		}
	}
}

/// # Panics
///
/// This function panics if the `num_grid_lines` calculation fails to cast to `usize`.
#[must_use]
pub fn compute_grid_line_info(
	min: f64,
	max: f64,
	distance_pixels: f64,
	grid_line_interval: &GridLineInterval,
) -> GridLineInfo {
	let range = max - min;
	let GridLineInterval { k, p } = *grid_line_interval;
	let interval = k * (10.0f64.powf(p));
	let interval_pixels = distance_pixels * (interval / range);
	let start = min - (min % interval) + if min > 0.0 { interval } else { 0.0 };
	let offset = start - min;
	let start_pixels = (offset / range) * distance_pixels;
	let num_grid_lines = ((range - offset) / interval).floor().to_usize().unwrap() + 1;
	GridLineInfo {
		interval,
		interval_pixels,
		k,
		num_grid_lines,
		p,
		start,
		start_pixels,
	}
}

pub struct ComputeXAxisGridLineInfoOptions<'a> {
	pub chart_width: f64,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub number_formatter: &'a NumberFormatter,
	pub x_axis_grid_line_interval: Option<GridLineInterval>,
	pub x_max: f64,
	pub x_min: f64,
}

/// # Panics
///
/// This function has two possible panics:
/// 1. The `grid_line_index` fails to cast from `usize` to `f64.`
/// 1. The `ctx` is unable to preform the `measure_text()` FFI method.
#[must_use]
pub fn compute_x_axis_grid_line_info(options: ComputeXAxisGridLineInfoOptions) -> GridLineInfo {
	let ComputeXAxisGridLineInfoOptions {
		chart_width,
		ctx,
		number_formatter,
		x_axis_grid_line_interval,
		x_max,
		x_min,
	} = options;
	if let Some(x_axis_grid_line_interval) = x_axis_grid_line_interval {
		return compute_grid_line_info(x_min, x_max, chart_width, &x_axis_grid_line_interval);
	}
	let mut x_axis_min_grid_line_distance = 1.0;
	loop {
		let x_axis_grid_line_interval =
			compute_grid_line_interval(x_min, x_max, chart_width, x_axis_min_grid_line_distance);
		let x_axis_grid_line_info =
			compute_grid_line_info(x_min, x_max, chart_width, &x_axis_grid_line_interval);
		let mut found_overlap = false;
		for grid_line_index in 0..x_axis_grid_line_info.num_grid_lines {
			let grid_line_value = x_axis_grid_line_info.start
				+ grid_line_index.to_f64().unwrap() * x_axis_grid_line_info.interval;
			let label = number_formatter.format(grid_line_value);
			let label_width = ctx.measure_text(&label).unwrap().width();
			if label_width > x_axis_grid_line_info.interval_pixels {
				x_axis_min_grid_line_distance = label_width;
				found_overlap = true;
				break;
			}
		}
		if !found_overlap {
			return x_axis_grid_line_info;
		}
	}
}

#[derive(Clone, Copy)]
pub struct ComputeYAxisGridLineInfoOptions<'a> {
	chart_height: f64,
	font_size: f64,
	y_axis_grid_line_interval: Option<&'a GridLineInterval>,
	y_max: f64,
	y_min: f64,
}

#[must_use]
pub fn compute_y_axis_grid_line_info(options: ComputeYAxisGridLineInfoOptions) -> GridLineInfo {
	let ComputeYAxisGridLineInfoOptions {
		y_axis_grid_line_interval,
		chart_height,
		font_size,
		y_max,
		y_min,
	} = options;
	let y_axis_grid_line_interval = y_axis_grid_line_interval
		.cloned()
		.unwrap_or_else(|| compute_grid_line_interval(y_min, y_max, chart_height, font_size));
	compute_grid_line_info(y_min, y_max, chart_height, &y_axis_grid_line_interval)
}

fn compute_axis_labels_max_width(
	ctx: &dom::CanvasRenderingContext2d,
	grid_line_info: &GridLineInfo,
	number_formatter: &NumberFormatter,
) -> f64 {
	(0..grid_line_info.num_grid_lines)
		.map(|grid_line_index| {
			let grid_line_value =
				grid_line_info.start + grid_line_index.to_f64().unwrap() * grid_line_info.interval;
			let label = number_formatter.format(grid_line_value);
			ctx.measure_text(&label).unwrap().width()
		})
		.max_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap()
}

pub struct DrawXAxisGridLinesOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub rect: Rect,
	pub x_axis_grid_line_info: GridLineInfo,
}

/// # Panics
///
///  This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
pub fn draw_x_axis_grid_lines(options: DrawXAxisGridLinesOptions) {
	let DrawXAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx,
		rect,
		x_axis_grid_line_info,
	} = options;
	for grid_line_index in 0..x_axis_grid_line_info.num_grid_lines {
		let grid_line_offset_pixels = x_axis_grid_line_info.start_pixels
			+ grid_line_index.to_f64().unwrap() * x_axis_grid_line_info.interval_pixels;
		let x = rect.x + grid_line_offset_pixels;
		ctx.begin_path();
		ctx.set_stroke_style(&chart_colors.grid_line_color.into());
		ctx.set_line_width(chart_config.axis_width);
		ctx.set_line_cap("square");
		ctx.move_to(x, rect.y);
		ctx.line_to(x, rect.y + rect.h);
		ctx.stroke();
	}
}

pub struct DrawXAxisOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub rect: Rect,
	pub y_axis_grid_line_info: &'a GridLineInfo,
}

/// # Panics
///
///  This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
pub fn draw_x_axis(options: &DrawXAxisOptions) {
	let DrawXAxisOptions {
		chart_colors,
		chart_config,
		rect,
		ctx,
		y_axis_grid_line_info,
	} = options;
	for grid_line_index in 0..y_axis_grid_line_info.num_grid_lines {
		let grid_line_value = y_axis_grid_line_info.start
			+ grid_line_index.to_f64().unwrap() * y_axis_grid_line_info.interval;
		if grid_line_value.abs() < f64::EPSILON {
			let grid_line_offset_pixels = y_axis_grid_line_info.start_pixels
				+ grid_line_index.to_f64().unwrap() * y_axis_grid_line_info.interval_pixels;
			let y = rect.y + rect.h - grid_line_offset_pixels;
			ctx.begin_path();
			ctx.set_stroke_style(&chart_colors.axis_color.into());
			ctx.set_line_width(chart_config.axis_width);
			ctx.set_line_cap("square");
			ctx.move_to(rect.x, y);
			ctx.line_to(rect.x + rect.w, y);
			ctx.stroke();
		}
	}
}

#[derive(Clone, Copy)]
pub struct DrawYAxisGridLinesOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub rect: Rect,
	pub y_axis_grid_line_info: &'a GridLineInfo,
}

/// # Panics
///
///  This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
pub fn draw_y_axis_grid_lines(options: DrawYAxisGridLinesOptions) {
	let DrawYAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx,
		rect,
		y_axis_grid_line_info,
	} = options;
	for grid_line_index in 0..y_axis_grid_line_info.num_grid_lines {
		let grid_line_offset_pixels = y_axis_grid_line_info.start_pixels
			+ grid_line_index.to_f64().unwrap() * y_axis_grid_line_info.interval_pixels;
		let y = rect.y + rect.h - grid_line_offset_pixels;
		ctx.begin_path();
		ctx.set_stroke_style(&chart_colors.grid_line_color.into());
		ctx.set_line_width(chart_config.axis_width);
		ctx.set_line_cap("square");
		ctx.move_to(rect.x, y);
		ctx.line_to(rect.x + rect.w, y);
		ctx.stroke();
	}
}

pub struct DrawYAxisOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub rect: Rect,
	pub x_axis_grid_line_info: &'a GridLineInfo,
}

/// # Panics
///
///  This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
pub fn draw_y_axis(options: &DrawYAxisOptions) {
	let DrawYAxisOptions {
		chart_colors,
		chart_config,
		ctx,
		rect,
		x_axis_grid_line_info,
	} = options;
	for grid_line_index in 0..x_axis_grid_line_info.num_grid_lines {
		let grid_line_value = x_axis_grid_line_info.start
			+ grid_line_index.to_f64().unwrap() * x_axis_grid_line_info.interval;
		if grid_line_value.abs() < f64::EPSILON {
			let grid_line_offset_pixels = x_axis_grid_line_info.start_pixels
				+ grid_line_index.to_f64().unwrap() * x_axis_grid_line_info.interval_pixels;
			let x = rect.x + grid_line_offset_pixels;
			ctx.begin_path();
			ctx.set_stroke_style(&chart_colors.axis_color.into());
			ctx.set_line_width(chart_config.axis_width);
			ctx.set_line_cap("square");
			ctx.move_to(x, rect.y);
			ctx.line_to(x, rect.y + rect.h);
			ctx.stroke();
		}
	}
}

pub struct DrawXAxisLabelsOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub rect: Rect,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub grid_line_info: GridLineInfo,
	pub labels: &'a Option<Vec<String>>,
	pub number_formatter: &'a NumberFormatter,
	pub width: f64,
}

/// # Panics
///
/// This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
/// It also panics if any FFI method calls to the `ctx` fail.
pub fn draw_x_axis_labels(options: DrawXAxisLabelsOptions) {
	let DrawXAxisLabelsOptions {
		chart_colors,
		rect,
		ctx,
		grid_line_info,
		labels,
		number_formatter,
		width,
	} = options;
	ctx.set_fill_style(&chart_colors.label_color.into());
	ctx.set_text_baseline("bottom");
	ctx.set_text_align("center");
	let mut previous_label_endpoint: Option<f64> = None;
	for grid_line_index in 0..grid_line_info.num_grid_lines {
		let grid_line_offset_pixels = grid_line_info.start_pixels
			+ grid_line_index.to_f64().unwrap() * grid_line_info.interval_pixels;
		let grid_line_value =
			grid_line_info.start + grid_line_index.to_f64().unwrap() * grid_line_info.interval;
		let label: Cow<str> = if let Some(labels) = &labels {
			labels.get(grid_line_index).unwrap().into()
		} else {
			number_formatter.format(grid_line_value).into()
		};
		// Do not draw the label if it will overlap the previous label.
		if let Some(previous_label_endpoint) = previous_label_endpoint {
			if grid_line_offset_pixels - ctx.measure_text(&label).unwrap().width() / 2.0
				< previous_label_endpoint
			{
				continue;
			}
		}
		// Do not draw the label if it will overflow the chart.
		if rect.x + grid_line_offset_pixels - ctx.measure_text(&label).unwrap().width() / 2.0 < 0.0
			|| rect.x + grid_line_offset_pixels + ctx.measure_text(&label).unwrap().width() / 2.0
				> width
		{
			break;
		}
		ctx.fill_text(&label, rect.x + grid_line_offset_pixels, rect.y + rect.h)
			.unwrap();
		// Set the endpoint value of the previous label. This is used to determine if the next label overlaps.
		previous_label_endpoint =
			Some(grid_line_offset_pixels + ctx.measure_text(&label).unwrap().width() / 2.0);
	}
}

#[derive(Clone, Copy)]
pub struct DrawYAxisLabelsOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub rect: Rect,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub font_size: f64,
	pub grid_line_info: &'a GridLineInfo,
	pub height: f64,
	pub number_formatter: &'a NumberFormatter,
}

/// # Panics
///
///  This function panics if unable to cast the `grid_line_index` from `usize` to `f64`.
pub fn draw_y_axis_labels(options: DrawYAxisLabelsOptions) {
	let DrawYAxisLabelsOptions {
		chart_colors,
		rect,
		ctx,
		font_size,
		grid_line_info,
		height,
		number_formatter,
	} = options;
	ctx.set_fill_style(&chart_colors.label_color.into());
	ctx.set_text_baseline("middle");
	ctx.set_text_align("right");
	for grid_line_index in 0..grid_line_info.num_grid_lines {
		let grid_line_offset_pixels = grid_line_info.start_pixels
			+ grid_line_index.to_f64().unwrap() * grid_line_info.interval_pixels;
		let grid_line_value =
			grid_line_info.start + grid_line_index.to_f64().unwrap() * grid_line_info.interval;
		let label = number_formatter.format(grid_line_value);
		if rect.y + rect.h - grid_line_offset_pixels - font_size / 2.0 < 0.0
			|| rect.y + rect.h - grid_line_offset_pixels + font_size / 2.0 > height
		{
			return;
		}
		ctx.fill_text(
			&label,
			rect.x + rect.w,
			rect.y + rect.h - grid_line_offset_pixels,
		)
		.unwrap();
	}
}

pub struct DrawXAxisTitleOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub rect: Rect,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub title: &'a str,
}

/// # Panics
///
/// This function panics if the FFI `ctx,fill_text()` method call fails.
pub fn draw_x_axis_title(options: &DrawXAxisTitleOptions) {
	let DrawXAxisTitleOptions {
		chart_colors,
		ctx,
		title,
		rect,
	} = options;
	let truncated_title = truncate_text(ctx, title, rect.w);
	ctx.save();
	ctx.set_text_align("center");
	ctx.set_text_baseline("bottom");
	ctx.set_fill_style(&chart_colors.title_color.into());
	ctx.fill_text(&truncated_title, rect.x + rect.w / 2.0, rect.y + rect.h)
		.unwrap();
	ctx.restore();
}

pub struct DrawYAxisTitleOptions<'a> {
	pub chart_colors: &'a ChartColors,
	pub rect: Rect,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub title: &'a str,
}

/// # Panics
///
/// This function panics if the FFI method calls to the `ctx` fail.
pub fn draw_y_axis_title(options: &DrawYAxisTitleOptions) {
	let DrawYAxisTitleOptions {
		chart_colors,
		ctx,
		title,
		rect,
	} = options;
	let truncated_title = truncate_text(ctx, title, rect.h);
	ctx.save();
	ctx.translate(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
		.unwrap();
	ctx.rotate(-std::f64::consts::PI / 2.0).unwrap();
	ctx.set_text_align("center");
	ctx.set_text_baseline("middle");
	ctx.set_fill_style(&chart_colors.title_color.into());
	ctx.fill_text(&truncated_title, 0.0, 0.0).unwrap();
	ctx.restore();
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy)]
pub struct DrawRoundedRectOptions<'a> {
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub fill_color: Option<&'a str>,
	pub radius: f64,
	pub rect: Rect,
	pub round_bottom_left: bool,
	pub round_bottom_right: bool,
	pub round_top_left: bool,
	pub round_top_right: bool,
	pub stroke_color: Option<&'a str>,
	pub stroke_width: Option<f64>,
}

/// # Panics
///
/// This function panics if the FFI `ctx` method calls fail.
pub fn draw_rounded_rect(options: DrawRoundedRectOptions) {
	let DrawRoundedRectOptions {
		ctx,
		fill_color,
		radius,
		rect,
		round_bottom_left,
		stroke_color,
		stroke_width,
		round_bottom_right,
		round_top_left,
		round_top_right,
	} = options;
	let Rect {
		mut h,
		mut w,
		mut x,
		mut y,
	} = rect;
	if h < 0.0 {
		h = -h;
		y -= h;
	}
	if w < 0.0 {
		w = -w;
		x -= w;
	}
	ctx.save();
	if let Some(stroke_width) = stroke_width {
		ctx.set_line_width(stroke_width);
	}
	if let Some(fill_color) = &fill_color {
		ctx.set_fill_style(&fill_color.to_owned().into());
	}
	if let Some(stroke_color) = &stroke_color {
		ctx.set_stroke_style(&stroke_color.to_owned().into());
	}
	ctx.begin_path();
	if round_top_left {
		ctx.move_to(x + radius, y);
	} else {
		ctx.move_to(x, y);
	}
	if round_top_right {
		ctx.line_to(x + w - radius, y);
		ctx.arc_to(x + w, y, x + w, y + radius, radius).unwrap();
	} else {
		ctx.line_to(x + w, y);
	}
	if round_bottom_right {
		ctx.line_to(x + w, y + h - radius);
		ctx.arc_to(x + w, y + h, x + w - radius, y + h, radius)
			.unwrap();
	} else {
		ctx.line_to(x + w, y + h);
	}
	if round_bottom_left {
		ctx.line_to(x + radius, y + h);
		ctx.arc_to(x, y + h, x, y + h - radius, radius).unwrap();
	} else {
		ctx.line_to(x, y + h);
	}
	if round_top_left {
		ctx.line_to(x, y + radius);
		ctx.arc_to(x, y, x + radius, y, radius).unwrap();
	} else {
		ctx.line_to(x, y);
	}
	if fill_color.is_some() {
		ctx.fill();
	}
	if stroke_color.is_some() {
		ctx.stroke();
	}
	ctx.restore();
}

fn truncate_text<'a>(
	ctx: &'a dom::CanvasRenderingContext2d,
	label: &'a str,
	width: f64,
) -> Cow<'a, str> {
	if ctx.measure_text(label).unwrap().width() < width {
		return label.into();
	}
	let mut longest_truncated_label = "...".to_owned();
	for i in 0..label.len() {
		let truncated_label = format!("{}...", &label[0..i]);
		let truncated_label_width = ctx.measure_text(&truncated_label).unwrap().width();
		if truncated_label_width < width {
			longest_truncated_label = truncated_label;
		} else {
			break;
		}
	}
	longest_truncated_label.into()
}
