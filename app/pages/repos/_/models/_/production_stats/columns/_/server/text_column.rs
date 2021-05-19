use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	colors::PRODUCTION_COLOR, date_window::DateWindow, metrics_row::MetricsRow,
	time::overall_chart_title,
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct TextColumn {
	pub alert: Option<String>,
	pub text_column_counts_section: TextColumnCountsSection,
	pub text_column_stats_section: TextColumnStatsSection,
	pub text_ngrams_section: TextColumnTokensSection,
}

impl Component for TextColumn {
	fn into_node(self) -> Node {
		fragment()
			.child(
				self.alert
					.map(|alert| ui::Alert::new(ui::Level::Danger).child(alert)),
			)
			.child(self.text_column_stats_section)
			.child(self.text_column_counts_section)
			.child(self.text_ngrams_section)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TextColumnStatsSection {
	pub column_name: String,
	pub date_window: DateWindow,
	pub ngram_row_counts: Vec<(String, u64)>,
}

impl Component for TextColumnStatsSection {
	fn into_node(self) -> Node {
		let overall_chart_series = vec![BarChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: self
				.ngram_row_counts
				.iter()
				.enumerate()
				.map(|(index, (ngram, row_count))| BarChartPoint {
					label: ngram.to_owned(),
					x: index.to_f64().unwrap(),
					y: Some(row_count.to_f64().unwrap()),
				})
				.collect(),
			title: Some("Production".to_owned()),
		}];
		let overall_distribution_chart_title = overall_chart_title(
			&self.date_window,
			format!("Distribution of Unique Values for {}", self.column_name),
		);
		ui::S2::new()
			.child(
				ui::Card::new().child(
					BarChart::new()
						.id("text_overall".to_owned())
						.series(Some(overall_chart_series))
						.title(Some(overall_distribution_chart_title))
						.x_axis_title(Some(self.column_name))
						.y_axis_title("Count".to_owned())
						.y_min(Some(0.0)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TextColumnCountsSection {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

impl Component for TextColumnCountsSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(
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
					)),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TextColumnTokensSection {
	pub text_ngrams_table: TextNGramsTable,
}

impl Component for TextColumnTokensSection {
	fn into_node(self) -> Node {
		if !self.text_ngrams_table.rows.is_empty() {
			Some(
				ui::S2::new()
					.child(ui::H2::new().child(format!(
						"Top {} Unique NGrams",
						self.text_ngrams_table.rows.len()
					)))
					.child(self.text_ngrams_table),
			)
		} else {
			None
		}
		.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TextNGramsTable {
	pub rows: Vec<TextNGramsTableRow>,
}

pub struct TextNGramsTableRow {
	pub ngram: String,
	pub count: usize,
}

impl Component for TextNGramsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("NGram"))
						.child(ui::TableHeaderCell::new().child("Count")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(ui::TableCell::new().child(row.ngram))
						.child(ui::TableCell::new().child(row.count.to_string()))
				})),
			)
			.into_node()
	}
}
