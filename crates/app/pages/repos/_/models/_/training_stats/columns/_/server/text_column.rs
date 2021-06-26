use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

pub struct TextColumn {
	pub name: String,
	pub ngram_count: usize,
	pub top_ngrams_chart_values: Vec<NGramStats>,
	pub ngrams_table_rows: Vec<NGramsTableRow>,
}

#[derive(Clone)]
pub struct NGramStats {
	pub ngram: String,
	pub occurrence_count: u64,
	pub row_count: u64,
}

#[derive(Clone)]
pub struct NGramsTableRow {
	pub ngram: String,
	pub count: u64,
}

impl Component for TextColumn {
	fn into_node(self) -> Node {
		let series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_string(),
			data: self
				.top_ngrams_chart_values
				.iter()
				.enumerate()
				.map(|(i, stats)| BarChartPoint {
					label: stats.ngram.clone(),
					x: i.to_f64().unwrap(),
					y: Some(stats.occurrence_count.to_f64().unwrap()),
				})
				.collect(),
			title: Some("NGram Count".to_owned()),
		}];
		let description = "Each value in this column was broken up into individual tokens. View the most frequent ngrams in the chart below.";
		let chart_title = format!(
			"{} Most Frequent NGrams",
			self.top_ngrams_chart_values.len()
		);
		let table_section_title = format!("Top {} NGrams", self.ngrams_table_rows.len());
		let table_section = ui::S2::new()
			.child(ui::H2::new().child(table_section_title))
			.child(
				ui::Table::new()
					.width("100%".to_owned())
					.child(
						ui::TableHeader::new()
							.child(ui::TableHeaderCell::new().child("Token"))
							.child(ui::TableHeaderCell::new().child("Count")),
					)
					.child(
						ui::TableBody::new().children(self.ngrams_table_rows.iter().map(
							|ngram_table_row| {
								ui::TableRow::new()
									.child(
										ui::TableCell::new()
											.child(ngram_table_row.ngram.to_string()),
									)
									.child(
										ui::TableCell::new()
											.child(ngram_table_row.count.to_string()),
									)
							},
						)),
					),
			);
		ui::S1::new()
			.child(ui::H1::new().child(self.name))
			.child(ui::S2::new().child(ui::P::new().child(description)))
			.child(
				ui::S2::new().child(
					ui::Card::new().child(Dehydrate::new(
						"ngram_histogram",
						BarChart::new()
							.series(Some(series))
							.title(Some(chart_title))
							.y_min(Some(0.0)),
					)),
				),
			)
			.child(ui::S2::new().child(table_section))
			.into_node()
	}
}
