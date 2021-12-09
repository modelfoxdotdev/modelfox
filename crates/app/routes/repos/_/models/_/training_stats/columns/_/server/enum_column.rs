use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::metrics_row::MetricsRow;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

pub struct EnumColumn {
	pub unique_values_chart_data: Option<Vec<(String, u64)>>,
	pub unique_values_table_rows: Option<Vec<(String, u64, f64)>>,
	pub invalid_count: u64,
	pub name: String,
	pub unique_count: u64,
}

impl Component for EnumColumn {
	fn into_node(self) -> Node {
		let name = self.name;
		ui::S1::new()
			.child(ui::H1::new().child(name.clone()))
			.child(
				ui::S2::new()
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"Unique Count".to_owned(),
								self.unique_count.to_string(),
							))
							.child(ui::NumberCard::new(
								"Invalid Count".to_owned(),
								self.invalid_count.to_string(),
							)),
					)
					.child(
						self.unique_values_chart_data
							.map(|unique_values_chart_data| EnumColumnUniqueValuesChart {
								name,
								unique_values_chart_data,
							}),
					)
					.child(
						self.unique_values_table_rows
							.map(|unique_values_table_rows| EnumColumnUniqueValuesTable {
								unique_values_table_rows,
							}),
					),
			)
			.into_node()
	}
}

pub struct EnumColumnUniqueValuesChart {
	name: String,
	unique_values_chart_data: Vec<(String, u64)>,
}

impl Component for EnumColumnUniqueValuesChart {
	fn into_node(self) -> Node {
		let data = self
			.unique_values_chart_data
			.iter()
			.enumerate()
			.map(|(i, (value, count))| BarChartPoint {
				label: value.clone(),
				x: i.to_f64().unwrap(),
				y: Some(count.to_f64().unwrap()),
			})
			.collect();
		let chart_series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data,
			title: Some("Unique Values".to_owned()),
		}];
		let enum_histogram_title = Some(format!("Histogram of Unique Values for {}", self.name));
		ui::Card::new()
			.child(Dehydrate::new(
				"enum_histogram",
				BarChart::new()
					.hide_legend(true)
					.series(chart_series)
					.title(enum_histogram_title)
					.x_axis_title(self.name)
					.y_axis_title("Count".to_owned())
					.y_min(0.0),
			))
			.into_node()
	}
}

pub struct EnumColumnUniqueValuesTable {
	unique_values_table_rows: Vec<(String, u64, f64)>,
}

impl Component for EnumColumnUniqueValuesTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Value"))
						.child(ui::TableHeaderCell::new().child("Count"))
						.child(ui::TableHeaderCell::new().child("Percent")),
				),
			)
			.child(
				ui::TableBody::new().children(self.unique_values_table_rows.iter().map(
					|(value, count, percent)| {
						ui::TableRow::new()
							.child(ui::TableCell::new().child(value.clone()))
							.child(ui::TableCell::new().child(count.to_string()))
							.child(ui::TableCell::new().child(ui::format_percent(*percent)))
					},
				)),
			)
			.into_node()
	}
}
