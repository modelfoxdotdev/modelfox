use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	colors::{PRODUCTION_COLOR, TRAINING_COLOR},
	date_window::DateWindow,
	metrics_row::MetricsRow,
	time::overall_chart_title,
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

pub struct EnumColumn {
	pub alert: Option<String>,
	pub counts_section: EnumColumnCountsSection,
	pub stats_section: EnumColumnStatsSection,
	pub unique_values_section: EnumColumnUniqueValuesSection,
	pub invalid_values_section: EnumColumnInvalidValuesSection,
}

pub struct EnumColumnOverallHistogramEntry {
	pub production_count: u64,
	pub production_fraction: Option<f32>,
	pub training_count: u64,
	pub training_fraction: f32,
}

impl Component for EnumColumn {
	fn into_node(self) -> Node {
		fragment()
			.child(
				self.alert
					.map(|alert| ui::Alert::new(ui::Level::Danger).child(alert)),
			)
			.child(self.stats_section)
			.child(self.counts_section)
			.child(self.unique_values_section)
			.child(self.invalid_values_section)
			.into_node()
	}
}

pub struct EnumColumnStatsSection {
	pub overall_chart_data: Vec<(String, EnumColumnOverallHistogramEntry)>,
	pub column_name: String,
	pub date_window: DateWindow,
}

impl Component for EnumColumnStatsSection {
	fn into_node(self) -> Node {
		let overall_chart_series = vec![
			BarChartSeries {
				color: TRAINING_COLOR.to_owned(),
				data: self
					.overall_chart_data
					.iter()
					.enumerate()
					.map(|(index, (label, value))| BarChartPoint {
						label: label.to_owned(),
						x: index.to_f64().unwrap(),
						y: Some(value.training_fraction.to_f64().unwrap()),
					})
					.collect(),
				title: Some("Training".to_owned()),
			},
			BarChartSeries {
				color: PRODUCTION_COLOR.to_owned(),
				data: self
					.overall_chart_data
					.iter()
					.enumerate()
					.map(|(index, (label, value))| BarChartPoint {
						label: label.to_owned(),
						x: index.to_f64().unwrap(),
						y: value
							.production_fraction
							.map(|production_fraction| production_fraction.to_f64().unwrap()),
					})
					.collect(),
				title: Some("Production".to_owned()),
			},
		];
		let overall_distribution_chart_title = overall_chart_title(
			&self.date_window,
			format!("Distribution of Unique Values for {}", self.column_name),
		);
		ui::Card::new()
			.child(Dehydrate::new(
				"enum_overall",
				BarChart::new()
					.series(overall_chart_series)
					.title(overall_distribution_chart_title)
					.x_axis_title(self.column_name)
					.y_axis_title("Percent".to_owned())
					.y_max(1.0)
					.y_min(0.0),
			))
			.into_node()
	}
}

pub struct EnumColumnCountsSection {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

impl Component for EnumColumnCountsSection {
	fn into_node(self) -> Node {
		MetricsRow::new()
			.child(ui::NumberCard::new(
				"Row Count".to_owned(),
				self.row_count.to_string(),
			))
			.child(ui::NumberCard::new(
				"Absent Count".to_owned(),
				self.absent_count.to_string(),
			))
			.child(ui::NumberCard::new(
				"Invalid Count".to_owned(),
				self.invalid_count.to_string(),
			))
			.into_node()
	}
}

pub struct EnumColumnUniqueValuesSection {
	pub enum_unique_values_table: EnumUniqueValuesTable,
}

impl Component for EnumColumnUniqueValuesSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Unique Values"))
			.child(EnumUniqueValuesTable {
				rows: self.enum_unique_values_table.rows,
			})
			.into_node()
	}
}

pub struct EnumUniqueValuesTable {
	pub rows: Vec<EnumUniqueValuesTableRow>,
}

pub struct EnumUniqueValuesTableRow {
	pub name: String,
	pub training_count: usize,
	pub production_count: usize,
	pub training_fraction: f32,
	pub production_fraction: Option<f32>,
}

impl Component for EnumUniqueValuesTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Value"))
						.child(ui::TableHeaderCell::new().child("Training Count"))
						.child(ui::TableHeaderCell::new().child("Production Count"))
						.child(ui::TableHeaderCell::new().child("Training Fraction"))
						.child(ui::TableHeaderCell::new().child("Production Fraction")),
				),
			)
			.child(ui::TableBody::new().children(self.rows.iter().map(|row| {
				ui::TableRow::new()
					.child(ui::TableCell::new().child(row.name.to_owned()))
					.child(ui::TableCell::new().child(row.training_count.to_string()))
					.child(ui::TableCell::new().child(row.production_count.to_string()))
					.child(ui::TableCell::new().child(ui::format_percent(row.training_fraction)))
					.child(
						ui::TableCell::new()
							.child(ui::format_option_percent(row.production_fraction)),
					)
			})))
			.into_node()
	}
}

pub struct EnumColumnInvalidValuesSection {
	pub enum_invalid_values_table: Option<EnumInvalidValuesTable>,
}

impl Component for EnumColumnInvalidValuesSection {
	fn into_node(self) -> Node {
		fragment()
			.child(
				self.enum_invalid_values_table
					.map(|enum_invalid_values_table| {
						ui::S2::new()
							.child(ui::H2::new().child("Invalid Values"))
							.child(enum_invalid_values_table)
					}),
			)
			.into_node()
	}
}

pub struct EnumInvalidValuesTable {
	pub rows: Vec<EnumInvalidValuesTableRow>,
}

pub struct EnumInvalidValuesTableRow {
	pub name: String,
	pub count: usize,
	pub production_fraction: f32,
}

impl Component for EnumInvalidValuesTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Value"))
						.child(ui::TableHeaderCell::new().child("Count"))
						.child(ui::TableHeaderCell::new().child("Production Fraction")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(ui::TableCell::new().child(row.name))
						.child(ui::TableCell::new().child(row.count.to_string()))
						.child(
							ui::TableCell::new().child(ui::format_percent(row.production_fraction)),
						)
				})),
			)
			.into_node()
	}
}
