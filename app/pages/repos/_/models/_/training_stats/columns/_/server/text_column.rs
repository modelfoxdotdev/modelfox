use html::{component, html, Props};
use num::ToPrimitive;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct TextColumnProps {
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

#[component]
pub fn TextColumn(props: TextColumnProps) {
	let series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_string(),
		data: props
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
	html! {
		<ui::S1>
			<ui::H1>{props.name}</ui::H1>
			<ui::S2>
				<ui::P>
					{description}
				</ui::P>
			</ui::S2>
			<ui::S2>
				<ui::Card>
					<BarChart
						id?="ngram_histogram"
						series?={Some(series)}
						title?={Some(format!("{} Most Frequent NGrams", props.top_ngrams_chart_values.len()))}
						y_min?={Some(0.0)}
					/>
				</ui::Card>
			</ui::S2>
			<ui::S2>
				<ui::H2>{format!("Top {} NGrams", props.ngrams_table_rows.len())}</ui::H2>
				<ui::Table width?="100%">
					<ui::TableHeader>
						<ui::TableHeaderCell>
							{"Token"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell>
							{"Count"}
						</ui::TableHeaderCell>
					</ui::TableHeader>
					<ui::TableBody>
					{props.ngrams_table_rows.iter().map(|ngram_table_row| html! {
						<ui::TableRow>
							<ui::TableCell>
								{ngram_table_row.ngram.to_string()}
							</ui::TableCell>
							<ui::TableCell>
								{ngram_table_row.count.to_string()}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
					</ui::TableBody>
				</ui::Table>
			</ui::S2>
		</ui::S1>
	}
}
