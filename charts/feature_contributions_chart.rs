use crate::{
	chart::{
		ActiveHoverRegion, ChartImpl, DrawChartOptions, DrawChartOutput, DrawOverlayOptions,
		HoverRegion,
	},
	common::{
		compute_x_axis_grid_line_info, draw_x_axis_grid_lines, draw_x_axis_labels,
		draw_x_axis_title, draw_y_axis_title, ComputeXAxisGridLineInfoOptions,
		DrawXAxisGridLinesOptions, DrawXAxisLabelsOptions, DrawXAxisTitleOptions,
		DrawYAxisTitleOptions, Point, Rect,
	},
	config::{ChartColors, ChartConfig},
	tooltip::{draw_tooltip, DrawTooltipOptions, TooltipLabel},
};
use num::ToPrimitive;
use tangram_number_formatter::NumberFormatter;
use web_sys as dom;

pub struct FeatureContributionsChart;

impl ChartImpl for FeatureContributionsChart {
	type Options = FeatureContributionsChartOptions;
	type OverlayInfo = FeatureContributionsChartOverlayInfo;
	type HoverRegionInfo = FeatureContributionsChartHoverRegionInfo;

	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo> {
		draw_feature_contributions_chart(options)
	}

	fn draw_overlay(
		options: DrawOverlayOptions<Self::Options, Self::OverlayInfo, Self::HoverRegionInfo>,
	) {
		draw_feature_contributions_chart_overlay(options)
	}
}

/// These are the options for displaying a feature contributions chart.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartOptions {
	pub include_x_axis_title: Option<bool>,
	pub include_y_axis_labels: Option<bool>,
	pub include_y_axis_title: Option<bool>,
	pub negative_color: String,
	pub number_formatter: NumberFormatter,
	pub positive_color: String,
	pub series: Vec<FeatureContributionsChartSeries>,
}

/// This is the configuration used across all feature contributions charts.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartConfig {
	pub arrow_depth: f64,
	pub bar_gap: f64,
	pub series_gap: f64,
	pub series_width: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct FeatureContributionsChartSeries {
	pub baseline: f64,
	pub baseline_label: String,
	pub output: f64,
	pub output_label: String,
	pub title: String,
	pub values: Vec<FeatureContributionsChartValue>,
	pub compressed_positive_values: Option<FeatureContributionsChartCompressedValues>,
	pub compressed_negative_values: Option<FeatureContributionsChartCompressedValues>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartValue {
	pub feature: String,
	pub value: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureContributionsChartCompressedValues {
	pub sum: f64,
	pub count: usize,
}

#[derive(Clone)]
pub struct FeatureContributionsChartHoverRegionInfo {
	rect: Rect,
	color: String,
	direction: FeatureContributionsBoxDirection,
	label: String,
	tooltip_origin_pixels: Point,
}

pub struct FeatureContributionsChartOverlayInfo {}

fn draw_feature_contributions_chart(
	options: DrawChartOptions<FeatureContributionsChartOptions>,
) -> DrawChartOutput<FeatureContributionsChartOverlayInfo, FeatureContributionsChartHoverRegionInfo>
{
	let DrawChartOptions {
		chart_colors,
		chart_config,
		ctx,
		options,
	} = options;
	let FeatureContributionsChartOptions {
		include_x_axis_title,
		include_y_axis_labels,
		include_y_axis_title,
		negative_color,
		positive_color,
		..
	} = options;
	let include_x_axis_title = include_x_axis_title.unwrap_or(false);
	let include_y_axis_labels = include_y_axis_labels.unwrap_or(false);
	let include_y_axis_title = include_y_axis_title.unwrap_or(false);

	let canvas = ctx.canvas().unwrap();
	let height = canvas.client_height().to_f64().unwrap();
	let width = canvas.client_width().to_f64().unwrap();
	let ChartConfig {
		bottom_padding,
		font_size,
		label_padding,
		left_padding,
		right_padding,
		top_padding,
		..
	} = chart_config;

	let annotations_padding = 80.0;
	let mut hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>> = Vec::new();

	let y_axis_labels_width = options
		.series
		.iter()
		.map(|series| ctx.measure_text(&series.title).unwrap().width())
		.max_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();

	let chart_width = width
		- (left_padding
			+ if include_y_axis_labels {
				y_axis_labels_width + label_padding
			} else {
				0.0
			} + right_padding
			+ if include_y_axis_title {
				font_size + label_padding
			} else {
				0.0
			} + annotations_padding);

	let chart_height = height
		- (top_padding
			+ font_size + label_padding
			+ font_size + label_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + bottom_padding);

	// Compress the series given the client chart width.
	let mut series = options.series.clone();
	compress_feature_contributions_chart_series(
		&mut series,
		CompressFeatureContributionsChartSeriesOptions {
			chart_width,
			min_box_width: chart_config.feature_contributions_arrow_depth * 2.0,
		},
	);

	// Compute the bounds.
	let (x_min, x_max) = compute_x_min_x_max(series.as_slice());

	let chart_rect = Rect {
		h: chart_height,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	let top_x_axis_title_rect = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: *top_padding,
	};
	if include_x_axis_title {
		draw_x_axis_title(DrawXAxisTitleOptions {
			chart_colors,
			rect: top_x_axis_title_rect,
			ctx,
			title: "Contributions",
		})
	}

	let top_x_axis_labels_rect = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			},
	};

	let bottom_x_axis_labels_rect = Rect {
		h: *font_size,
		w: chart_width,
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + if include_y_axis_labels {
			y_axis_labels_width + label_padding
		} else {
			0.0
		} + annotations_padding,
		y: top_padding
			+ font_size + label_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + chart_height
			+ label_padding,
	};

	let y_axis_labels_rect = Rect {
		h: chart_height,
		w: if include_y_axis_labels {
			y_axis_labels_width
		} else {
			0.0
		},
		x: left_padding
			+ if include_y_axis_title {
				label_padding + font_size
			} else {
				0.0
			},
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	let y_axis_titles_rect = Rect {
		h: chart_height,
		w: if include_y_axis_title {
			*font_size
		} else {
			0.0
		},
		x: *left_padding,
		y: top_padding
			+ if include_x_axis_title {
				label_padding + font_size
			} else {
				0.0
			} + font_size
			+ label_padding,
	};

	if include_y_axis_title {
		draw_y_axis_title(DrawYAxisTitleOptions {
			chart_colors,
			rect: y_axis_titles_rect,
			ctx,
			title: "Class",
		});
	}

	// Compute the grid line info.
	let x_axis_grid_line_info = compute_x_axis_grid_line_info(ComputeXAxisGridLineInfoOptions {
		chart_width: chart_rect.w,
		ctx,
		number_formatter: &options.number_formatter,
		x_axis_grid_line_interval: None,
		x_max,
		x_min,
	});

	draw_x_axis_grid_lines(DrawXAxisGridLinesOptions {
		rect: chart_rect,
		ctx,
		x_axis_grid_line_info: x_axis_grid_line_info.clone(),
		chart_colors,
		chart_config,
	});

	draw_x_axis_labels(DrawXAxisLabelsOptions {
		chart_colors,
		rect: top_x_axis_labels_rect,
		ctx,
		grid_line_info: x_axis_grid_line_info.clone(),
		labels: &None,
		number_formatter: &options.number_formatter,
		width,
	});

	draw_x_axis_labels(DrawXAxisLabelsOptions {
		chart_colors,
		rect: bottom_x_axis_labels_rect,
		ctx,
		grid_line_info: x_axis_grid_line_info,
		labels: &None,
		number_formatter: &options.number_formatter,
		width,
	});

	let categories = options
		.series
		.iter()
		.map(|series| &series.title)
		.collect::<Vec<_>>();
	if include_y_axis_labels {
		draw_feature_contributions_chart_y_axis_labels(
			DrawFeatureContributionsChartYAxisLabelsOptions {
				rect: y_axis_labels_rect,
				categories: &categories,
				ctx,
				chart_config,
			},
		);
	}

	// Draw the series separators.
	for i in 1..categories.len() {
		let y = chart_rect.y
			+ i.to_f64().unwrap() * chart_config.feature_contributions_series_height
			+ (i - 1).to_f64().unwrap() * chart_config.feature_contributions_series_gap
			+ chart_config.feature_contributions_series_gap / 2.0;
		ctx.save();
		ctx.set_stroke_style(&chart_colors.grid_line_color.into());
		ctx.move_to(chart_rect.x, y);
		ctx.line_to(chart_rect.x + chart_rect.w, y);
		ctx.stroke();
		ctx.restore();
	}

	let value_width_multiplier = chart_rect.w / (x_max - x_min);
	for (series_index, series) in series.iter().enumerate() {
		let mut sum_positives = series
			.values
			.iter()
			.filter(|value| value.value > 0.0)
			.map(|value| value.value)
			.sum::<f64>();
		if let Some(sum_compressed_positives) = &series.compressed_positive_values {
			sum_positives += sum_compressed_positives.sum
		};
		let min = series.baseline.min(series.output);
		let max = series.baseline + sum_positives;
		let width = max - min;
		let box_height = (chart_config.feature_contributions_series_height
			- chart_config.feature_contributions_bar_gap)
			/ 2.0;
		let rect = Rect {
			h: chart_config.feature_contributions_series_height,
			w: width * value_width_multiplier,
			x: chart_rect.x + (min - x_min) * value_width_multiplier,
			y: chart_rect.y
				+ (chart_config.feature_contributions_series_gap
					+ chart_config.feature_contributions_series_height)
					* series_index.to_f64().unwrap(),
		};
		let output = draw_feature_contribution_series(DrawFeatureContributionSeriesOptions {
			chart_config,
			rect,
			box_height,
			ctx,
			negative_color,
			positive_color,
			series,
			value_width_multiplier,
		});
		hover_regions.extend(output.hover_regions);
	}

	DrawChartOutput {
		hover_regions,
		overlay_info: FeatureContributionsChartOverlayInfo {},
	}
}

struct DrawFeatureContributionSeriesOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	box_height: f64,
	ctx: &'a dom::CanvasRenderingContext2d,
	negative_color: &'a str,
	positive_color: &'a str,
	series: &'a FeatureContributionsChartSeries,
	value_width_multiplier: f64,
}

struct DrawFeatureContributionsSeriesOutput {
	hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>>,
}

fn draw_feature_contribution_series(
	options: DrawFeatureContributionSeriesOptions,
) -> DrawFeatureContributionsSeriesOutput {
	let mut hover_regions: Vec<HoverRegion<FeatureContributionsChartHoverRegionInfo>> = Vec::new();
	let DrawFeatureContributionSeriesOptions {
		rect,
		box_height,
		ctx,
		negative_color,
		positive_color,
		series,
		value_width_multiplier,
		chart_config,
	} = options;
	let min = series.baseline.min(series.output);

	// Draw the positive boxes which start at the baseline and go to the max, ending with the remaining features rect.
	let mut positive_values: Vec<FeatureContributionsChartValue> = series
		.values
		.iter()
		.cloned()
		.filter(|value| value.value > 0.0)
		.collect();
	positive_values.sort_unstable_by(|a, b| a.value.partial_cmp(&b.value).unwrap().reverse());
	let mut x = rect.x + (series.baseline - min) * value_width_multiplier;
	// Draw the baseline value and label.
	ctx.set_text_baseline("bottom");
	ctx.set_text_align("right");
	ctx.fill_text(
		"baseline",
		x - chart_config.label_padding,
		rect.y + box_height / 2.0,
	)
	.unwrap();
	ctx.set_text_baseline("top");
	ctx.set_text_align("right");
	ctx.fill_text(
		&series.baseline_label,
		x - chart_config.label_padding,
		rect.y + box_height / 2.0,
	)
	.unwrap();
	for positive_value in positive_values {
		let feature_contribution_value = positive_value;
		let width = feature_contribution_value.value * value_width_multiplier;
		let value_rect = Rect {
			h: box_height,
			w: width,
			x,
			y: rect.y,
		};
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: value_rect,
			color: positive_color.to_owned(),
			ctx,
			direction: FeatureContributionsBoxDirection::Negative,
			label: feature_contribution_value.feature.clone(),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: value_rect,
				color: positive_color.to_owned(),
				direction: FeatureContributionsBoxDirection::Negative,
				label: feature_contribution_value.feature.clone(),
				tooltip_origin_pixels: Point {
					x: value_rect.x + value_rect.w / 2.0,
					y: value_rect.y,
				},
			},
		));
		x += width;
	}
	if let Some(too_small_positive_values) = series.compressed_positive_values.as_ref() {
		let n_remaining_features = too_small_positive_values.count;
		let feature_contribution_value = too_small_positive_values;
		let width = feature_contribution_value.sum * value_width_multiplier;
		let remaining_features_rect = Rect {
			h: box_height,
			w: width,
			x,
			y: rect.y,
		};
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: remaining_features_rect,
			color: format!("{}33", positive_color),
			ctx,
			direction: FeatureContributionsBoxDirection::Negative,
			label: format!("{} other features", n_remaining_features),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: remaining_features_rect,
				color: format!("{}33", positive_color),
				direction: FeatureContributionsBoxDirection::Negative,
				label: format!("{} other features", n_remaining_features),
				tooltip_origin_pixels: Point {
					x: remaining_features_rect.x + remaining_features_rect.w / 2.0,
					y: remaining_features_rect.y,
				},
			},
		));
		x += width;
	}

	// Draw the negative boxes which start at the max and go to the output, starting with the remaining features rect.
	let y = rect.y + box_height + chart_config.feature_contributions_bar_gap;
	let mut negative_values: Vec<FeatureContributionsChartValue> = series
		.values
		.iter()
		.cloned()
		.filter(|value| value.value < 0.0)
		.collect();
	negative_values.sort_unstable_by(|a, b| a.value.partial_cmp(&b.value).unwrap().reverse());
	if let Some(too_small_negative_values) = series.compressed_negative_values.as_ref() {
		let n_remaining_features = too_small_negative_values.count;
		let feature_contribution_value = too_small_negative_values;
		let width = feature_contribution_value.sum * value_width_multiplier;
		let remaining_features_rect = Rect {
			h: box_height,
			w: width,
			x,
			y,
		};
		x += width;
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: remaining_features_rect,
			color: format!("{}33", negative_color),
			ctx,
			direction: FeatureContributionsBoxDirection::Positive,
			label: format!("{} other features", n_remaining_features),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: remaining_features_rect,
				color: format!("{}33", negative_color),
				direction: FeatureContributionsBoxDirection::Positive,
				label: format!("{} other features", n_remaining_features),
				tooltip_origin_pixels: Point {
					x: remaining_features_rect.x + remaining_features_rect.w / 2.0,
					y: remaining_features_rect.y,
				},
			},
		))
	}
	for negative_value in negative_values {
		let feature_contribution_value = negative_value;
		let width = feature_contribution_value.value * value_width_multiplier;
		let value_rect = Rect {
			h: box_height,
			w: width,
			x,
			y,
		};
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: value_rect,
			color: negative_color.to_owned(),
			ctx,
			direction: FeatureContributionsBoxDirection::Positive,
			label: feature_contribution_value.feature.clone(),
			chart_config,
		});
		hover_regions.push(feature_contributions_chart_hover_region(
			FeatureContributionsChartHoverRegionOptions {
				rect: value_rect,
				color: negative_color.to_owned(),
				direction: FeatureContributionsBoxDirection::Positive,
				label: feature_contribution_value.feature.clone(),
				tooltip_origin_pixels: Point {
					x: value_rect.x + value_rect.w / 2.0,
					y: value_rect.y,
				},
			},
		));
		x += width;
	}
	// Draw the output value and label.
	ctx.set_text_baseline("bottom");
	ctx.fill_text(
		"output",
		x - chart_config.label_padding,
		rect.y + box_height + chart_config.feature_contributions_bar_gap + box_height / 2.0,
	)
	.unwrap();
	ctx.set_text_baseline("top");
	ctx.fill_text(
		&series.output_label,
		x - chart_config.label_padding,
		rect.y + box_height + chart_config.feature_contributions_bar_gap + box_height / 2.0,
	)
	.unwrap();

	DrawFeatureContributionsSeriesOutput { hover_regions }
}

struct DrawFeatureContributionsChartYAxisLabelsOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	categories: &'a [&'a String],
	ctx: &'a dom::CanvasRenderingContext2d,
}

fn draw_feature_contributions_chart_y_axis_labels(
	options: DrawFeatureContributionsChartYAxisLabelsOptions,
) {
	let DrawFeatureContributionsChartYAxisLabelsOptions {
		chart_config,
		rect,
		categories,
		ctx,
	} = options;
	ctx.set_text_align("end");
	for (i, label) in categories.iter().enumerate() {
		let label_offset = chart_config.feature_contributions_series_height / 2.0
			+ (chart_config.feature_contributions_series_gap
				+ chart_config.feature_contributions_series_height)
				* i.to_f64().unwrap();
		ctx.set_text_baseline("middle");
		ctx.fill_text(label, rect.x + rect.w, rect.y + label_offset)
			.unwrap();
	}
}

fn draw_feature_contributions_chart_overlay(
	options: DrawOverlayOptions<
		FeatureContributionsChartOptions,
		FeatureContributionsChartOverlayInfo,
		FeatureContributionsChartHoverRegionInfo,
	>,
) {
	let DrawOverlayOptions {
		active_hover_regions,
		ctx,
		overlay_div,
		chart_config,
		chart_colors,
		..
	} = options;
	draw_feature_contribution_tooltips(DrawFeatureContributionTooltipsOptions {
		chart_colors,
		chart_config,
		active_hover_regions,
		overlay_div,
	});
	for active_hover_region in active_hover_regions {
		draw_feature_contribution_box(DrawFeatureContributionBoxOptions {
			rect: active_hover_region.info.rect,
			color: "#00000022".to_owned(),
			ctx,
			direction: active_hover_region.info.direction,
			label: "".to_owned(),
			chart_config,
		});
	}
}

struct DrawFeatureContributionTooltipsOptions<'a> {
	chart_colors: &'a ChartColors,
	chart_config: &'a ChartConfig,
	active_hover_regions: &'a [ActiveHoverRegion<FeatureContributionsChartHoverRegionInfo>],
	overlay_div: &'a dom::HtmlElement,
}

fn draw_feature_contribution_tooltips(options: DrawFeatureContributionTooltipsOptions) {
	let DrawFeatureContributionTooltipsOptions {
		chart_colors,
		chart_config,
		active_hover_regions,
		overlay_div,
	} = options;
	for active_hover_region in active_hover_regions {
		let label = TooltipLabel {
			color: active_hover_region.info.color.clone(),
			text: active_hover_region.info.label.clone(),
		};
		draw_tooltip(DrawTooltipOptions {
			center_horizontal: Some(true),
			container: overlay_div,
			labels: vec![label],
			origin: active_hover_region.info.tooltip_origin_pixels,
			chart_colors,
			chart_config,
			flip_y_offset: None,
		});
	}
}

struct FeatureContributionsChartHoverRegionOptions {
	rect: Rect,
	color: String,
	direction: FeatureContributionsBoxDirection,
	label: String,
	tooltip_origin_pixels: Point,
}

fn feature_contributions_chart_hover_region(
	options: FeatureContributionsChartHoverRegionOptions,
) -> HoverRegion<FeatureContributionsChartHoverRegionInfo> {
	let FeatureContributionsChartHoverRegionOptions {
		rect,
		color,
		direction,
		label,
		tooltip_origin_pixels,
	} = options;
	HoverRegion {
		distance: Box::new(move |x, y| (rect.x - x).powi(2) + (rect.y - y).powi(2)),
		hit_test: Box::new(move |x, y| {
			x > rect.x.min(rect.x + rect.w)
				&& x < rect.x.max(rect.x + rect.w)
				&& y > rect.y && y < rect.y + rect.h
		}),
		info: FeatureContributionsChartHoverRegionInfo {
			rect,
			color,
			direction,
			label,
			tooltip_origin_pixels,
		},
	}
}

struct DrawFeatureContributionBoxOptions<'a> {
	chart_config: &'a ChartConfig,
	rect: Rect,
	color: String,
	ctx: &'a dom::CanvasRenderingContext2d,
	direction: FeatureContributionsBoxDirection,
	label: String,
}

#[derive(Clone, Copy)]
enum FeatureContributionsBoxDirection {
	Positive,
	Negative,
}

fn draw_feature_contribution_box(options: DrawFeatureContributionBoxOptions) {
	let DrawFeatureContributionBoxOptions {
		chart_config,
		rect,
		color,
		ctx,
		direction,
		label,
	} = options;

	let text_padding = 4.0;
	let arrow_depth = if let FeatureContributionsBoxDirection::Negative = direction {
		chart_config.feature_contributions_arrow_depth
	} else {
		-chart_config.feature_contributions_arrow_depth
	};
	let width = rect.w;

	ctx.save();
	ctx.set_stroke_style(&color.clone().into());
	ctx.set_fill_style(&color.into());
	ctx.set_line_width(1.0);
	ctx.set_line_cap("butt");

	ctx.begin_path();
	ctx.move_to(rect.x, rect.y);
	let draw_end_arrow = true;
	let draw_start_arrow = true;

	// Draw the endpoint.
	if draw_end_arrow {
		ctx.line_to(rect.x + width - arrow_depth, rect.y);
		ctx.line_to(rect.x + width, rect.y + rect.h / 2.0);
		ctx.line_to(rect.x + width - arrow_depth, rect.y + rect.h);
	} else {
		ctx.line_to(rect.x + width, rect.y);
		ctx.line_to(rect.x + width, rect.y + rect.h);
	}

	// Draw the startpoint.
	if draw_start_arrow {
		ctx.line_to(rect.x, rect.y + rect.h);
		ctx.line_to(rect.x + arrow_depth, rect.y + rect.h / 2.0);
		ctx.line_to(rect.x, rect.y);
	} else {
		ctx.line_to(rect.x, rect.y + rect.h);
		ctx.line_to(rect.x, rect.y);
	}

	ctx.fill();

	let label_width = ctx.measure_text(&label).unwrap().width();
	ctx.set_text_baseline("middle");
	ctx.set_text_align("center");
	ctx.set_fill_style(&"#fff".into());

	let max_label_width =
		rect.w.abs() - text_padding - chart_config.feature_contributions_arrow_depth * 2.0;
	if label_width <= max_label_width {
		ctx.fill_text(
			&label,
			rect.x + (rect.w + arrow_depth) / 2.0,
			rect.y + rect.h / 2.0,
		)
		.unwrap();
	}

	ctx.restore();
}

#[derive(Default)]
pub struct CompressFeatureContributionsChartSeriesOptions {
	/// Determine the width of each box if the chart were drawn at this width.
	pub chart_width: f64,
	/// Compress all boxes whose width would be smaller than this.
	pub min_box_width: f64,
}

pub fn compress_feature_contributions_chart_series(
	series: &mut [FeatureContributionsChartSeries],
	options: CompressFeatureContributionsChartSeriesOptions,
) {
	let (x_min, x_max) = compute_x_min_x_max(series);
	let chart_width = options.chart_width;
	let min_box_width = options.min_box_width;
	let value_width_multiplier = chart_width / (x_max - x_min);
	let max_feature_contribution_value_to_compress = min_box_width / value_width_multiplier;
	series.iter_mut().for_each(|feature_contribution_series| {
		let mut compressed_negative_values = FeatureContributionsChartCompressedValues {
			sum: feature_contribution_series
				.compressed_negative_values
				.as_ref()
				.map(|compressed_negative_values| compressed_negative_values.sum)
				.unwrap_or(0.0),
			count: feature_contribution_series
				.compressed_negative_values
				.as_ref()
				.map(|compressed_negative_values| compressed_negative_values.count)
				.unwrap_or(0),
		};
		let mut compressed_positive_values = FeatureContributionsChartCompressedValues {
			sum: feature_contribution_series
				.compressed_positive_values
				.as_ref()
				.map(|compressed_positive_values| compressed_positive_values.sum)
				.unwrap_or(0.0),
			count: feature_contribution_series
				.compressed_positive_values
				.as_ref()
				.map(|compressed_positive_values| compressed_positive_values.count)
				.unwrap_or(0),
		};
		feature_contribution_series.values.retain(|value| {
			if value.value.abs() > max_feature_contribution_value_to_compress {
				true
			} else {
				if value.value < 0.0 {
					compressed_negative_values.count += 1;
					compressed_negative_values.sum += value.value;
				} else {
					compressed_positive_values.count += 1;
					compressed_positive_values.sum += value.value;
				}
				false
			}
		});
		feature_contribution_series.compressed_negative_values = Some(compressed_negative_values);
		feature_contribution_series.compressed_positive_values = Some(compressed_positive_values);
	});
}

fn compute_x_min_x_max(series: &[FeatureContributionsChartSeries]) -> (f64, f64) {
	let min_baseline = series
		.iter()
		.map(|series| series.baseline)
		.min_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	let min_output = series
		.iter()
		.map(|series| series.output)
		.min_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	let x_min = min_baseline.min(min_output);
	let x_max = series
		.iter()
		.map(|series| {
			let sum_of_positive_values = series
				.values
				.iter()
				.filter_map(|value| {
					if value.value > 0.0 {
						Some(value.value)
					} else {
						None
					}
				})
				.sum::<f64>();
			let sum_of_too_small_positive_values = series
				.compressed_positive_values
				.as_ref()
				.map(|t| t.sum)
				.unwrap_or(0.0);
			series.baseline + sum_of_positive_values + sum_of_too_small_positive_values
		})
		.max_by(|a, b| a.partial_cmp(b).unwrap())
		.unwrap();
	(x_min, x_max)
}
