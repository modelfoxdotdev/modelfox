use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	date_window::DateWindow, metrics_row::MetricsRow, time::overall_chart_title,
	tokens::PRODUCTION_COLOR,
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct TextColumnProps {
	pub alert: Option<String>,
	pub text_column_counts_section_props: TextColumnCountsSectionProps,
	pub text_column_stats_section_props: TextColumnStatsSectionProps,
	pub text_ngrams_section_props: TextColumnTokensSectionProps,
}

#[component]
pub fn TextColumn(props: TextColumnProps) {
	html! {
		<>
			{props.alert.map(|alert| html! {
				<ui::Alert level={ui::Level::Danger}>
					{alert}
				</ui::Alert>
			})}
			// <TextColumnStatsSection {props.text_column_stats_section_props} />
			<TextColumnCountsSection {props.text_column_counts_section_props} />
			<TextNGramsSection {props.text_ngrams_section_props} />
		</>
	}
}

#[derive(Props)]
pub struct TextColumnStatsSectionProps {
	pub column_name: String,
	pub date_window: DateWindow,
	pub ngram_row_counts: Vec<(String, u64)>,
}

#[component]
fn TextColumnStatsSection(props: TextColumnStatsSectionProps) {
	let overall_chart_series = vec![BarChartSeries {
		color: PRODUCTION_COLOR.to_owned(),
		data: props
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
		&props.date_window,
		format!("Distribution of Unique Values for {}", props.column_name),
	);
	html! {
		<ui::S2>
			<ui::Card>
				<BarChart
					id?="text_overall"
					series?={Some(overall_chart_series)}
					title?={Some(overall_distribution_chart_title)}
					x_axis_title?={Some(props.column_name)}
					y_axis_title?="Count"
					y_min?={Some(0.0)}
				/>
			</ui::Card>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct TextColumnCountsSectionProps {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

#[component]
fn TextColumnCountsSection(props: TextColumnCountsSectionProps) {
	html! {
		<ui::S2>
			<MetricsRow>
				<ui::NumberCard
					title="Row Count"
					value={props.row_count.to_string()}
				/>
				<ui::NumberCard
					title="Absent Count"
					value={props.absent_count.to_string()}
				/>
				<ui::NumberCard
					title="Invalid Count"
					value={props.invalid_count.to_string()}
				/>
			</MetricsRow>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct TextColumnTokensSectionProps {
	pub text_ngrams_table_props: TextNGramsTableProps,
}

#[component]
fn TextNGramsSection(props: TextColumnTokensSectionProps) {
	if !props.text_ngrams_table_props.rows.is_empty() {
		html! {
			<ui::S2>
				<ui::H2>{format!("Top {} Unique NGrams", props.text_ngrams_table_props.rows.len())}</ui::H2>
				<TextNGramsTable {props.text_ngrams_table_props} />
			</ui::S2>
		}
	} else {
		html! { <></> }
	}
}

#[derive(Props)]
pub struct TextNGramsTableProps {
	pub rows: Vec<TextNGramsTableRow>,
}

pub struct TextNGramsTableRow {
	pub ngram: String,
	pub count: usize,
}

#[component]
fn TextNGramsTable(props: TextNGramsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"NGram"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Count"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.into_iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
							{row.ngram}
						</ui::TableCell>
						<ui::TableCell>
							{row.count.to_string()}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
