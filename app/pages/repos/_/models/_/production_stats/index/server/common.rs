use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	column_type::ColumnType,
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	time::{interval_chart_title, overall_chart_title},
	tokens::ColumnTypeToken,
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;
use tangram_zip::zip;

pub struct PredictionCountChartEntry {
	pub count: u64,
	pub label: String,
}

pub struct ProductionTrainingHistogram {
	pub production: Vec<(String, u64)>,
	pub training: Vec<(String, u64)>,
}

pub struct ClassifierChartEntry {
	pub label: String,
	pub histogram: ProductionTrainingHistogram,
}

#[derive(ComponentBuilder)]
pub struct ClassificationProductionStatsIntervalChart {
	pub chart_data: Vec<ClassifierChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

impl Component for ClassificationProductionStatsIntervalChart {
	fn into_node(self) -> Node {
		let color_options = vec![
			ui::colors::GREEN,
			ui::colors::BLUE,
			ui::colors::INDIGO,
			ui::colors::PURPLE,
			ui::colors::PINK,
			ui::colors::RED,
			ui::colors::ORANGE,
			ui::colors::YELLOW,
		];
		let title = interval_chart_title(&self.date_window_interval, "Prediction Stats".to_owned());
		let classes = self.chart_data[0]
			.histogram
			.production
			.iter()
			.cloned()
			.map(|(class, _)| class)
			.collect::<Vec<_>>();
		let series = classes
			.iter()
			.enumerate()
			.map(|(index, class)| {
				let color = color_options[index % color_options.len()].to_owned();
				BarChartSeries {
					color,
					data: self
						.chart_data
						.iter()
						.enumerate()
						.map(|(entry_index, entry)| BarChartPoint {
							label: entry.label.to_owned(),
							x: entry_index.to_f64().unwrap(),
							y: Some(entry.histogram.production[index].1.to_f64().unwrap()),
						})
						.collect::<Vec<_>>(),
					title: Some(class.to_owned()),
				}
			})
			.collect::<Vec<_>>();
		BarChart::new()
			.id("histogram_intervals".to_owned())
			.series(Some(series))
			.title(Some(title))
			.y_min(Some(0.0))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ClassificationProductionStatsChart {
	pub chart_data: ClassifierChartEntry,
	pub date_window: DateWindow,
}

impl Component for ClassificationProductionStatsChart {
	fn into_node(self) -> Node {
		let color_options = vec![
			ui::colors::GREEN,
			ui::colors::BLUE,
			ui::colors::INDIGO,
			ui::colors::PURPLE,
			ui::colors::PINK,
			ui::colors::RED,
			ui::colors::ORANGE,
			ui::colors::YELLOW,
		];
		let classes = self
			.chart_data
			.histogram
			.production
			.iter()
			.cloned()
			.map(|(class, _)| class)
			.collect::<Vec<_>>();
		let title = overall_chart_title(&self.date_window, "Prediction Stats".to_owned());
		let series = zip!(classes.iter(), self.chart_data.histogram.production.iter())
			.enumerate()
			.map(|(index, (class, entry))| {
				let color = color_options[index % color_options.len()].to_owned();
				BarChartSeries {
					color,
					data: vec![BarChartPoint {
						label: self.chart_data.label.to_owned(),
						x: 0.0,
						y: Some(entry.1.to_f64().unwrap()),
					}],
					title: Some(class.to_owned()),
				}
			})
			.collect::<Vec<_>>();
		BarChart::new()
			.id("histogram_overall".to_owned())
			.series(Some(series))
			.title(Some(title))
			.y_min(Some(0.0))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct PredictionCountChart {
	pub chart_data: Vec<PredictionCountChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

impl Component for PredictionCountChart {
	fn into_node(self) -> Node {
		let prediction_count_chart_series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: self
				.chart_data
				.into_iter()
				.enumerate()
				.map(|(index, entry)| BarChartPoint {
					label: entry.label,
					x: index.to_f64().unwrap(),
					y: Some(entry.count.to_f64().unwrap()),
				})
				.collect::<Vec<_>>(),
			title: Some("Prediction Count".to_owned()),
		}];
		let prediction_count_title = interval_chart_title(
			&self.date_window_interval,
			"Total Prediction Count".to_owned(),
		);
		BarChart::new()
			.id("prediction_count".to_owned())
			.series(Some(prediction_count_chart_series))
			.title(Some(prediction_count_title))
			.y_min(Some(0.0))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ColumnStatsTable {
	pub rows: Vec<ColumnStatsTableRow>,
}

pub struct ColumnStatsTableRow {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub alert: Option<String>,
	pub href: Option<String>,
	pub name: String,
	pub column_type: ColumnType,
}

impl Component for ColumnStatsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Status"))
						.child(ui::TableHeaderCell::new().child("Column"))
						.child(ui::TableHeaderCell::new().child("Type"))
						.child(ui::TableHeaderCell::new().child("Absent Count"))
						.child(ui::TableHeaderCell::new().child("Invalid Count")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					let link_or_label = if row.href.is_some() {
						ui::Link::new()
							.href(row.href.unwrap())
							.child(row.name)
							.into_node()
					} else {
						span().child(row.name).into_node()
					};
					ui::TableRow::new()
						.child(ui::TableCell::new().child(if row.alert.is_some() {
							ui::AlertIcon::new(row.alert.unwrap(), ui::Level::Danger).child("!")
						} else {
							ui::AlertIcon::new("All good".to_owned(), ui::Level::Success).child("✓")
						}))
						.child(ui::TableCell::new().child(link_or_label))
						.child(ui::TableCell::new().child(ColumnTypeToken::new(row.column_type)))
						.child(ui::TableCell::new().child(row.absent_count.to_string()))
						.child(ui::TableCell::new().child(row.invalid_count.to_string()))
				})),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct DateWindowSelectForm {
	pub date_window: DateWindow,
}

impl Component for DateWindowSelectForm {
	fn into_node(self) -> Node {
		ui::Form::new()
			.child(DateWindowSelectField::new(self.date_window))
			.child(
				noscript().child(
					ui::Button::new()
						.button_type(Some(ui::ButtonType::Submit))
						.child("Submit"),
				),
			)
			.into_node()
	}
}
