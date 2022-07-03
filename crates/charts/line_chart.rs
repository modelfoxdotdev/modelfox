use crate::{
	chart::{
		ActiveHoverRegion, ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions,
		HoverRegion,
	},
	common::{
		compute_rects, compute_x_axis_grid_line_info, draw_x_axis, draw_x_axis_grid_lines,
		draw_x_axis_labels, draw_x_axis_title, draw_y_axis, draw_y_axis_grid_lines,
		draw_y_axis_labels, draw_y_axis_title, ComputeRectsOptions, ComputeRectsOutput,
		ComputeXAxisGridLineInfoOptions, DrawXAxisGridLinesOptions, DrawXAxisLabelsOptions,
		DrawXAxisOptions, DrawXAxisTitleOptions, DrawYAxisGridLinesOptions, DrawYAxisLabelsOptions,
		DrawYAxisOptions, DrawYAxisTitleOptions, GridLineInterval, Point, Rect,
	},
	config::ChartConfig,
	tooltip::{draw_tooltip, DrawTooltipOptions, TooltipLabel},
};
use itertools::Itertools;
use modelfox_finite::Finite;
use modelfox_number_formatter::NumberFormatter;
use num::ToPrimitive;
use wasm_bindgen::JsValue;
use web_sys as dom;

pub struct LineChart;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LineChartOptions {
	pub hide_legend: Option<bool>,
	pub labels: Option<Vec<String>>,
	pub number_formatter: NumberFormatter,
	pub series: Vec<LineChartSeries>,
	pub should_draw_x_axis_labels: Option<bool>,
	pub should_draw_y_axis_labels: Option<bool>,
	pub title: Option<String>,
	pub x_axis_grid_line_interval: Option<GridLineInterval>,
	pub x_axis_title: Option<String>,
	pub x_max: Option<Finite<f64>>,
	pub x_min: Option<Finite<f64>>,
	pub y_axis_grid_line_interval: Option<GridLineInterval>,
	pub y_axis_title: Option<String>,
	pub y_max: Option<Finite<f64>>,
	pub y_min: Option<Finite<f64>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineChartSeries {
	pub color: String,
	pub data: Vec<LineChartPoint>,
	pub line_style: Option<LineStyle>,
	pub point_style: Option<PointStyle>,
	pub title: Option<String>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineChartPoint {
	pub x: Finite<f64>,
	pub y: Option<Finite<f64>>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum LineStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "solid")]
	Solid,
	#[serde(rename = "dashed")]
	Dashed,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum PointStyle {
	#[serde(rename = "hidden")]
	Hidden,
	#[serde(rename = "circle")]
	Circle,
}

pub struct LineChartOverlayInfo {
	chart_rect: Rect,
	n_series: usize,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

#[derive(Clone)]
pub struct LineChartHoverRegionInfo {
	color: String,
	point: Point,
	point_label: Option<String>,
	point_value: f64,
	series_index: usize,
	series_title: Option<String>,
	tooltip_origin_pixels: Point,
}

impl ChartImpl for LineChart {
	type Options = LineChartOptions;
	type OverlayInfo = LineChartOverlayInfo;
	type HoverRegionInfo = LineChartHoverRegionInfo;

	fn draw_chart(
		options: &DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_line_chart(options)
	}

	fn draw_overlay(
		options: &DrawOverlayOptions<Self::Options, Self::OverlayInfo, Self::HoverRegionInfo>,
	) {
		draw_line_chart_overlay(options);
	}
}

#[allow(clippy::too_many_lines)]
fn draw_line_chart(
	options: &DrawChartOptions<LineChartOptions>,
) -> DrawChartOutput<LineChartOverlayInfo, LineChartHoverRegionInfo> {
	let DrawChartOptions {
		chart_colors,
		chart_config,
		options,
		ctx,
		..
	} = options;
	let LineChartOptions {
		labels,
		series,
		x_axis_grid_line_interval,
		x_axis_title,
		y_axis_grid_line_interval,
		y_axis_title,
		..
	} = &options;
	let canvas = ctx.canvas().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let mut hover_regions: Vec<HoverRegion<LineChartHoverRegionInfo>> = Vec::new();

	// Compute the bounds.
	let x_min: f64 = options
		.x_min
		.unwrap_or_else(|| {
			options
				.series
				.iter()
				.flat_map(|series| series.data.iter().map(|point| point.x))
				.min_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap()
		})
		.get();
	let x_max: f64 = options
		.x_max
		.unwrap_or_else(|| {
			options
				.series
				.iter()
				.flat_map(|series| series.data.iter().map(|point| point.x))
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap()
		})
		.get();
	let y_min: f64 = options
		.y_min
		.unwrap_or_else(|| {
			options
				.series
				.iter()
				.flat_map(|series| series.data.iter().map(|point| point.y))
				.flatten()
				.min_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap()
		})
		.get();
	let y_max: f64 = options
		.y_max
		.unwrap_or_else(|| {
			options
				.series
				.iter()
				.flat_map(|series| series.data.iter().map(|point| point.y))
				.flatten()
				.max_by(|a, b| a.partial_cmp(b).unwrap())
				.unwrap()
		})
		.get();
	let (y_min, y_max) = if (y_max - y_min).abs() < f64::EPSILON {
		(y_min, y_min + 1.0)
	} else {
		(y_min, y_max)
	};
	let (x_min, x_max) = if (x_max - x_min).abs() < f64::EPSILON {
		(x_min, x_min + 1.0)
	} else {
		(x_min, x_max)
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
		x_axis_grid_line_interval: x_axis_grid_line_interval.as_ref(),
		y_axis_grid_line_interval: y_axis_grid_line_interval.as_ref(),
		y_max,
		y_min,
	});

	// Compute the grid line info.
	let x_axis_grid_line_info = compute_x_axis_grid_line_info(ComputeXAxisGridLineInfoOptions {
		chart_width: chart_rect.w,
		ctx,
		number_formatter: &options.number_formatter,
		x_axis_grid_line_interval: x_axis_grid_line_interval.clone(),
		x_max,
		x_min,
	});

	draw_x_axis_grid_lines(DrawXAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		x_axis_grid_line_info: x_axis_grid_line_info.clone(),
	});

	draw_y_axis_grid_lines(DrawYAxisGridLinesOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		y_axis_grid_line_info: &y_axis_grid_line_info,
	});

	draw_x_axis(&DrawXAxisOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		y_axis_grid_line_info: &y_axis_grid_line_info,
	});

	draw_y_axis(&DrawYAxisOptions {
		chart_colors,
		chart_config,
		ctx,
		rect: chart_rect,
		x_axis_grid_line_info: &x_axis_grid_line_info,
	});

	// Draw the X axis labels.
	if options.should_draw_x_axis_labels.unwrap_or(true) {
		draw_x_axis_labels(DrawXAxisLabelsOptions {
			chart_colors,
			rect: x_axis_labels_rect,
			ctx,
			grid_line_info: x_axis_grid_line_info,
			labels,
			number_formatter: &options.number_formatter,
			width,
		});
	}

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
		});
	}

	// Draw the X axis title.
	if let Some(x_axis_title) = x_axis_title {
		draw_x_axis_title(&DrawXAxisTitleOptions {
			chart_colors,
			rect: x_axis_title_rect,
			ctx,
			title: x_axis_title,
		});
	}

	// Draw the Y axis title.
	if let Some(y_axis_title) = y_axis_title {
		draw_y_axis_title(&DrawYAxisTitleOptions {
			chart_colors,
			rect: y_axis_title_rect,
			ctx,
			title: y_axis_title,
		});
	}

	// Draw the lines.
	for series in series.iter() {
		draw_line(DrawLineOptions {
			chart_rect,
			chart_config,
			ctx,
			series,
			x_max,
			x_min,
			y_max,
			y_min,
		});
	}

	let max_point_count = series
		.iter()
		.map(|series| series.data.len())
		.max()
		.unwrap()
		.to_f64()
		.unwrap();
	let should_draw_points = chart_rect.w / max_point_count > 2.0 * chart_config.point_radius;

	// Draw the points.
	if should_draw_points {
		for series in series.iter() {
			for point in &series.data {
				if let Some(y) = point.y {
					draw_point(DrawPointOptions {
						chart_rect,
						color: &series.color,
						ctx,
						point: Point {
							x: point.x.get(),
							y: y.get(),
						},
						point_style: series.point_style.unwrap_or(PointStyle::Circle),
						radius: chart_config.point_radius,
						x_max,
						x_min,
						y_max,
						y_min,
					});
				}
			}
		}
	}

	// Compute the hover regions.
	let has_multiple_series = series.len() > 1;
	for (series_index, series) in series.iter().enumerate() {
		for (point_index, point) in series.data.iter().enumerate() {
			let point = match point.y {
				Some(y) => Point {
					x: point.x.get(),
					y: y.get(),
				},
				None => continue,
			};
			let point_pixels = point_to_pixels(PointToPixelsOptions {
				chart_rect,
				point,
				x_max,
				x_min,
				y_max,
				y_min,
			});
			let tooltip_target_radius = chart_config.tooltip_target_radius;
			let point_label = labels
				.as_ref()
				.map(|labels| labels.get(point_index).unwrap().clone());
			let hover_region = HoverRegion {
				distance: Box::new(move |x, y| {
					(point_pixels.x - x).powi(2) + (point_pixels.y - y).powi(2)
				}),
				hit_test: Box::new(move |x, y| {
					x > point_pixels.x - tooltip_target_radius
						&& x < point_pixels.x + tooltip_target_radius
						&& y > point_pixels.y - tooltip_target_radius
						&& y < point_pixels.y + tooltip_target_radius
				}),
				info: LineChartHoverRegionInfo {
					color: series.color.clone(),
					point,
					point_label,
					point_value: point.y,
					series_index,
					series_title: if has_multiple_series {
						series.title.clone()
					} else {
						None
					},
					tooltip_origin_pixels: point_pixels,
				},
			};
			hover_regions.push(hover_region);
		}
	}

	let overlay_info = LineChartOverlayInfo {
		chart_rect,
		n_series: options.series.len(),
		x_max,
		x_min,
		y_max,
		y_min,
	};

	DrawChartOutput {
		hover_regions,
		overlay_info,
	}
}

#[derive(Clone, Copy)]
struct DrawPointOptions<'a> {
	chart_rect: Rect,
	color: &'a str,
	ctx: &'a dom::CanvasRenderingContext2d,
	point: Point,
	point_style: PointStyle,
	radius: f64,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn draw_point(options: DrawPointOptions) {
	let DrawPointOptions {
		chart_rect,
		color,
		ctx,
		point,
		point_style,
		radius,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	if let PointStyle::Hidden = point_style {
		return;
	}
	let point_pixels = point_to_pixels(PointToPixelsOptions {
		chart_rect,
		point,
		x_max,
		x_min,
		y_max,
		y_min,
	});
	ctx.begin_path();
	ctx.set_fill_style(&color.into());
	ctx.arc(
		point_pixels.x,
		point_pixels.y,
		radius,
		0.0,
		2.0 * std::f64::consts::PI,
	)
	.unwrap();
	ctx.fill();
}

#[derive(Clone, Copy)]
struct DrawLineOptions<'a> {
	chart_rect: Rect,
	chart_config: &'a ChartConfig,
	ctx: &'a dom::CanvasRenderingContext2d,
	series: &'a LineChartSeries,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

#[allow(clippy::too_many_lines)]
fn draw_line(options: DrawLineOptions) {
	let DrawLineOptions {
		chart_rect,
		chart_config,
		ctx,
		series,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	if let Some(LineStyle::Hidden) = &series.line_style {
		return;
	}
	ctx.save();
	ctx.begin_path();
	ctx.set_stroke_style(&series.color.as_str().into());
	if let Some(LineStyle::Dashed) = &series.line_style {
		ctx.set_line_dash(&JsValue::from_serde(&[4, 4]).unwrap())
			.unwrap();
	}
	if series.data.len() < 2 {
		return;
	}
	let contiguous_point_ranges = get_contiguous_point_ranges(&series.data);
	for contiguous_point_range in contiguous_point_ranges {
		let series = &series.data[contiguous_point_range];
		let first_point = series[0];
		let first_point = Point {
			x: first_point.x.get(),
			y: first_point.y.unwrap().get(),
		};
		let first_point_pixels = point_to_pixels(PointToPixelsOptions {
			chart_rect,
			point: first_point,
			x_max,
			x_min,
			y_max,
			y_min,
		});
		ctx.move_to(first_point_pixels.x, first_point_pixels.y);
		let mut cp1 = first_point;
		for (previous_point, point, next_point) in series.iter().tuple_windows() {
			let previous_point = Point {
				x: previous_point.x.get(),
				y: previous_point.y.unwrap().get(),
			};
			let next_point = Point {
				x: next_point.x.get(),
				y: next_point.y.unwrap().get(),
			};
			let point = Point {
				x: point.x.get(),
				y: point.y.unwrap().get(),
			};
			let (cp2, next_cp1) = interpolate_spline(InterpolateSplineOptions {
				previous_point,
				point,
				next_point,
				tension: chart_config.spline_tension,
			});
			let cp1_pixels = point_to_pixels(PointToPixelsOptions {
				chart_rect,
				point: cp1,
				x_max,
				x_min,
				y_max,
				y_min,
			});
			let cp2_pixels = point_to_pixels(PointToPixelsOptions {
				chart_rect,
				point: Point { x: cp2.x, y: cp2.y },
				x_max,
				x_min,
				y_max,
				y_min,
			});
			let point_pixels = point_to_pixels(PointToPixelsOptions {
				chart_rect,
				point,
				x_max,
				x_min,
				y_max,
				y_min,
			});
			ctx.bezier_curve_to(
				cp1_pixels.x,
				cp1_pixels.y,
				cp2_pixels.x,
				cp2_pixels.y,
				point_pixels.x,
				point_pixels.y,
			);
			cp1 = next_cp1;
		}
		let last_point = series[series.len() - 1];
		let last_point = Point {
			x: last_point.x.get(),
			y: last_point.y.unwrap().get(),
		};
		let last_point_pixels = point_to_pixels(PointToPixelsOptions {
			chart_rect,
			point: last_point,
			x_max,
			x_min,
			y_max,
			y_min,
		});
		let cp1_pixels = point_to_pixels(PointToPixelsOptions {
			chart_rect,
			point: cp1,
			x_max,
			x_min,
			y_max,
			y_min,
		});
		ctx.bezier_curve_to(
			cp1_pixels.x,
			cp1_pixels.y,
			last_point_pixels.x,
			last_point_pixels.y,
			last_point_pixels.x,
			last_point_pixels.y,
		);
		ctx.stroke();
	}
	ctx.restore();
}

#[derive(Clone, Copy)]
struct InterpolateSplineOptions {
	next_point: Point,
	point: Point,
	previous_point: Point,
	tension: f64,
}

#[allow(clippy::similar_names)]
fn interpolate_spline(options: InterpolateSplineOptions) -> (Point, Point) {
	let InterpolateSplineOptions {
		next_point,
		point,
		previous_point,
		tension,
	} = options;
	let d01 = ((point.x - previous_point.x).powi(2) + (point.y - previous_point.y).powi(2)).sqrt();
	let d12 = ((point.x - next_point.x).powi(2) + (point.y - next_point.y).powi(2)).sqrt();
	let m01 = (tension * d01) / (d01 + d12);
	let m12 = (tension * d12) / (d01 + d12);
	let cp1 = Point {
		x: point.x - m01 * (next_point.x - previous_point.x),
		y: point.y - m01 * (next_point.y - previous_point.y),
	};
	let cp2 = Point {
		x: point.x + m12 * (next_point.x - previous_point.x),
		y: point.y + m12 * (next_point.y - previous_point.y),
	};
	(cp1, cp2)
}

#[allow(clippy::too_many_lines)]
fn draw_line_chart_overlay(
	options: &DrawOverlayOptions<LineChartOptions, LineChartOverlayInfo, LineChartHoverRegionInfo>,
) {
	let DrawOverlayOptions {
		chart_colors,
		chart_config,
		active_hover_regions,
		ctx,
		options,
		overlay_info,
		overlay_div,
	} = options;
	let LineChartOverlayInfo {
		chart_rect,
		n_series,
		x_max,
		x_min,
		y_max,
		y_min,
	} = &overlay_info;
	let mut closest_active_hover_region_for_series: Vec<
		Option<ActiveHoverRegion<LineChartHoverRegionInfo>>,
	> = Vec::with_capacity(*n_series);
	for _ in 0..*n_series {
		closest_active_hover_region_for_series.push(None);
	}
	for active_hover_region in *active_hover_regions {
		// Update the current closest active hover region for the series corresponding to this active_hover_region's series index.
		let current_closest_active_hover_region_for_series = closest_active_hover_region_for_series
			.get_mut(active_hover_region.info.series_index)
			.unwrap();
		*current_closest_active_hover_region_for_series = Some(
			match current_closest_active_hover_region_for_series.take() {
				None => active_hover_region.clone(),
				Some(current_closest_active_hover_region_for_series) => {
					if active_hover_region.distance
						< current_closest_active_hover_region_for_series.distance
					{
						active_hover_region.clone()
					} else {
						current_closest_active_hover_region_for_series
					}
				}
			},
		);
	}
	let tooltips: Vec<TooltipLabel> = closest_active_hover_region_for_series
		.iter()
		.filter_map(|active_hover_region| {
			active_hover_region.as_ref().map(|active_hover_region| {
				let series_title = &active_hover_region.info.series_title;
				let point_label = active_hover_region.info.point_label.clone();
				let point_label = point_label.unwrap_or_else(|| {
					options
						.number_formatter
						.format(active_hover_region.info.point.x)
				});
				let point_value = options
					.number_formatter
					.format(active_hover_region.info.point_value);
				let text = if let Some(series_title) = series_title {
					format!("{} ({}, {})", series_title, point_label, point_value)
				} else {
					format!("({}, {})", point_label, point_value)
				};
				TooltipLabel {
					color: active_hover_region.info.color.clone(),
					text,
				}
			})
		})
		.collect();
	let closest_active_hover_region = active_hover_regions
		.iter()
		.min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
	if let Some(closest_active_hover_region) = closest_active_hover_region {
		let tooltip_origin = closest_active_hover_region.info.tooltip_origin_pixels;
		draw_crosshairs(DrawCrosshairsOptions {
			chart_rect: *chart_rect,
			crosshairs_color: chart_colors.crosshairs_color.to_owned(),
			ctx,
			origin: tooltip_origin,
		});
		draw_tooltip(DrawTooltipOptions {
			center_horizontal: None,
			chart_colors,
			chart_config,
			container: overlay_div,
			flip_y_offset: None,
			labels: tooltips,
			origin: tooltip_origin,
		});
	}
	for active_hover_region in closest_active_hover_region_for_series
		.iter()
		.filter_map(std::option::Option::as_ref)
	{
		let point = active_hover_region.info.point;
		draw_point(DrawPointOptions {
			chart_rect: *chart_rect,
			color: &active_hover_region.info.color,
			ctx,
			point,
			point_style: PointStyle::Circle,
			radius: chart_config.point_radius,
			x_max: *x_max,
			x_min: *x_min,
			y_max: *y_max,
			y_min: *y_min,
		});
		draw_point(DrawPointOptions {
			chart_rect: *chart_rect,
			color: "#00000022",
			ctx,
			point,
			point_style: PointStyle::Circle,
			radius: chart_config.point_radius,
			x_max: *x_max,
			x_min: *x_min,
			y_max: *y_max,
			y_min: *y_min,
		});
	}
}

struct DrawCrosshairsOptions<'a> {
	chart_rect: Rect,
	crosshairs_color: String,
	ctx: &'a dom::CanvasRenderingContext2d,
	origin: Point,
}

fn draw_crosshairs(options: DrawCrosshairsOptions) {
	let DrawCrosshairsOptions {
		chart_rect,
		crosshairs_color,
		ctx,
		origin,
	} = options;
	ctx.save();
	ctx.begin_path();
	ctx.set_line_dash(&JsValue::from_serde(&[4, 4]).unwrap())
		.unwrap();
	ctx.set_stroke_style(&crosshairs_color.into());
	ctx.move_to(origin.x, chart_rect.y);
	ctx.line_to(origin.x, chart_rect.y + chart_rect.h);
	ctx.move_to(chart_rect.x, origin.y);
	ctx.line_to(chart_rect.x + chart_rect.w, origin.y);
	ctx.stroke();
	ctx.restore();
}

#[derive(Clone, Copy)]
struct PointToPixelsOptions {
	chart_rect: Rect,
	point: Point,
	x_max: f64,
	x_min: f64,
	y_max: f64,
	y_min: f64,
}

fn point_to_pixels(options: PointToPixelsOptions) -> Point {
	let PointToPixelsOptions {
		chart_rect,
		point,
		x_max,
		x_min,
		y_max,
		y_min,
	} = options;
	Point {
		x: chart_rect.x
			+ (-x_min / (x_max - x_min)) * chart_rect.w
			+ (point.x / (x_max - x_min)) * chart_rect.w,
		y: chart_rect.y + chart_rect.h
			- (-y_min / (y_max - y_min)) * chart_rect.h
			- (point.y / (y_max - y_min)) * chart_rect.h,
	}
}

fn get_contiguous_point_ranges(data: &[LineChartPoint]) -> Vec<std::ops::Range<usize>> {
	let first_point_pos = data.iter().position(|point| point.y.is_some());
	let first_point_pos = match first_point_pos {
		Some(first_point_pos) => first_point_pos,
		None => return Vec::new(),
	};
	let mut contiguous_point_ranges = Vec::new();
	let mut contiguous_region_start = first_point_pos;
	let mut end = first_point_pos;
	while end < data.len() {
		let point = data[end];
		if point.y.is_some() {
			// continue adding to the contiguous point block
			end += 1;
		} else {
			// commit the previous point block
			contiguous_point_ranges.push(contiguous_region_start..end);
			// find the next finite region start point
			contiguous_region_start = end + 1;
			while contiguous_region_start < data.len() {
				let point = data[contiguous_region_start];
				if point.y.is_some() {
					break;
				}
				contiguous_region_start += 1;
			}
			end = contiguous_region_start;
		}
	}
	if contiguous_region_start < end {
		contiguous_point_ranges.push(contiguous_region_start..end);
	}
	contiguous_point_ranges
}

#[test]
fn test_contiguous_point_ranges() {
	let data = vec![
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: None,
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(2.0).unwrap(),
			y: Some(Finite::new(2.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(3.0).unwrap(),
			y: None,
		},
		LineChartPoint {
			x: Finite::new(4.0).unwrap(),
			y: None,
		},
	];
	assert_eq!(get_contiguous_point_ranges(&data), vec![1..3]);
	let data = vec![
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: Some(Finite::new(0.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(2.0).unwrap(),
			y: Some(Finite::new(2.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(3.0).unwrap(),
			y: None,
		},
		LineChartPoint {
			x: Finite::new(4.0).unwrap(),
			y: None,
		},
		LineChartPoint {
			x: Finite::new(5.0).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(6.0).unwrap(),
			y: Some(Finite::new(2.0).unwrap()),
		},
	];
	assert_eq!(get_contiguous_point_ranges(&data), vec![0..3, 5..7]);
}
