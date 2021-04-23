pub struct ChartConfig {
	pub axis_width: f64,
	pub bar_gap: f64,
	pub bar_group_gap: f64,
	pub bar_stroke_width: f64,
	pub bottom_padding: f64,
	pub feature_contributions_arrow_depth: f64,
	pub feature_contributions_bar_gap: f64,
	pub feature_contributions_series_gap: f64,
	pub feature_contributions_series_height: f64,
	pub font: &'static str,
	pub font_size: f64,
	pub label_padding: f64,
	pub left_padding: f64,
	pub max_corner_radius: f64,
	pub point_halo_radius: f64,
	pub point_radius: f64,
	pub right_padding: f64,
	pub spline_tension: f64,
	pub tooltip_border_radius: f64,
	pub tooltip_padding: f64,
	pub tooltip_shadow_blur: f64,
	pub tooltip_target_radius: f64,
	pub top_padding: f64,
}

impl Default for ChartConfig {
	fn default() -> ChartConfig {
		ChartConfig {
			axis_width: 2.0,
			bar_gap: 2.0,
			bar_group_gap: 4.0,
			bar_stroke_width: 2.0,
			bottom_padding: 8.0,
			feature_contributions_arrow_depth: 4.0,
			feature_contributions_bar_gap: 10.0,
			feature_contributions_series_gap: 20.0,
			feature_contributions_series_height: 100.0,
			font: "14px JetBrains Mono",
			font_size: 14.0,
			label_padding: 8.0,
			left_padding: 8.0,
			max_corner_radius: 8.0,
			point_halo_radius: 8.0,
			point_radius: 4.0,
			right_padding: 8.0,
			spline_tension: 0.0,
			tooltip_border_radius: 4.0,
			tooltip_padding: 4.0,
			tooltip_shadow_blur: 2.0,
			tooltip_target_radius: 5.0,
			top_padding: 8.0,
		}
	}
}

pub struct ChartColors {
	pub axis_color: &'static str,
	pub border_color: &'static str,
	pub crosshairs_color: &'static str,
	pub grid_line_color: &'static str,
	pub label_color: &'static str,
	pub text_color: &'static str,
	pub title_color: &'static str,
	pub tooltip_background_color: &'static str,
	pub tooltip_shadow_color: &'static str,
}

pub const LIGHT_CHART_COLORS: ChartColors = ChartColors {
	axis_color: "#BBBBBB",
	border_color: "#EEEEEE",
	crosshairs_color: "#666666",
	grid_line_color: "#EEEEEE",
	label_color: "#666666",
	text_color: "#222222",
	title_color: "#222222",
	tooltip_background_color: "#FFFFFF",
	tooltip_shadow_color: "rgba(0, 0, 0, .1)",
};

pub const DARK_CHART_COLORS: ChartColors = ChartColors {
	axis_color: "#AAAAAA",
	border_color: "#333333",
	crosshairs_color: "#AAAAAA",
	grid_line_color: "#222222",
	label_color: "#888888",
	text_color: "#EEEEEE",
	title_color: "#EEEEEE",
	tooltip_background_color: "#333333",
	tooltip_shadow_color: "rgba(0, 0, 0, .1)",
};
