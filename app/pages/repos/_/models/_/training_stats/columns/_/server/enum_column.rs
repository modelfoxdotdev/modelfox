use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct EnumColumnProps {
	pub unique_values_chart_data: Option<Vec<(String, u64)>>,
	pub unique_values_table_rows: Option<Vec<(String, u64, f64)>>,
	pub invalid_count: u64,
	pub name: String,
	pub unique_count: u64,
}

#[component]
pub fn EnumColumn(props: EnumColumnProps) {
	let name = props.name;
	html! {
		<ui::S1>
			<ui::H1>{name.clone()}</ui::H1>
			<ui::S2>
				<MetricsRow>
					<ui::NumberCard
						title="Unique Count"
						value={props.unique_count.to_string()}
					/>
					<ui::NumberCard
						title="Invalid Count"
						value={props.invalid_count.to_string()}
					/>
				</MetricsRow>
				{props.unique_values_chart_data.map(|unique_values_chart_data| {
					html! {
						<EnumColumnUniqueValuesChart
							name={name}
							unique_values_chart_data={unique_values_chart_data}
						/>
					}
				})}
				{props.unique_values_table_rows.map(|unique_values_table_rows| {
					html! {
						<EnumColumnUniqueValuesTable
							unique_values_table_rows={unique_values_table_rows}
						/>
					}
				})}
			</ui::S2>
		</ui::S1>
	}
}

#[derive(Props)]
pub struct EnumColumnUniqueValuesChartProps {
	name: String,
	unique_values_chart_data: Vec<(String, u64)>,
}

#[component]
fn EnumColumnUniqueValuesChart(props: EnumColumnUniqueValuesChartProps) {
	let data = props
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
	html! {
		<ui::Card>
			<BarChart
				id?="enum_histogram"
				hide_legend?={Some(true)}
				series?={Some(chart_series)}
				title?={Some(format!("Histogram of Unique Values for {}", props.name))}
				x_axis_title?={Some(props.name)}
				y_axis_title?="Count"
				y_min?={Some(0.0)}
			/>
		</ui::Card>
	}
}

#[derive(Props)]
pub struct EnumColumnUniqueValuesTableProps {
	unique_values_table_rows: Vec<(String, u64, f64)>,
}

#[component]
fn EnumColumnUniqueValuesTable(props: EnumColumnUniqueValuesTableProps) {
	html! {
			<ui::Table width?="100%">
				<ui::TableHeader>
					<ui::TableRow>
						<ui::TableHeaderCell>
							{"Value"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell>
							{"Count"}
						</ui::TableHeaderCell>
						<ui::TableHeaderCell>
							{"Percent"}
						</ui::TableHeaderCell>
					</ui::TableRow>
				</ui::TableHeader>
				<ui::TableBody>
					{props.unique_values_table_rows.iter().map(|(value, count, percent)| html! {
						<ui::TableRow>
							<ui::TableCell>
								{value.clone()}
							</ui::TableCell>
							<ui::TableCell>
								{count.to_string()}
							</ui::TableCell>
							<ui::TableCell>
								{ui::format_percent(*percent)}
							</ui::TableCell>
						</ui::TableRow>
					}).collect::<Vec<_>>()}
				</ui::TableBody>
			</ui::Table>
	}
}
