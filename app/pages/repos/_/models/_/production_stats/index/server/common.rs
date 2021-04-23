use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	column_type::ColumnType,
	date_window::{DateWindow, DateWindowInterval},
};
use tangram_app_common::{
	date_window_select_field::DateWindowSelectField,
	time::{interval_chart_title, overall_chart_title},
	tokens::column_type_token,
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

#[derive(Props)]
pub struct ClassificationProductionStatsIntervalChartProps {
	pub chart_data: Vec<ClassifierChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

#[component]
pub fn ClassificationProductionStatsIntervalChart(
	props: ClassificationProductionStatsIntervalChartProps,
) {
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
	let title = interval_chart_title(&props.date_window_interval, "Prediction Stats".to_owned());
	let classes = props.chart_data[0]
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
				data: props
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
	html! {
		<BarChart
			id?="histogram_intervals"
			series?={Some(series)}
			title?={Some(title)}
			y_min?={Some(0.0)}
		/>
	}
}

#[derive(Props)]
pub struct ClassificationProductionStatsChartProps {
	pub chart_data: ClassifierChartEntry,
	pub date_window: DateWindow,
}

#[component]
pub fn ClassificationProductionStatsChart(props: ClassificationProductionStatsChartProps) {
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
	let classes = props
		.chart_data
		.histogram
		.production
		.iter()
		.cloned()
		.map(|(class, _)| class)
		.collect::<Vec<_>>();
	let title = overall_chart_title(&props.date_window, "Prediction Stats".to_owned());
	let series = zip!(classes.iter(), props.chart_data.histogram.production.iter())
		.enumerate()
		.map(|(index, (class, entry))| {
			let color = color_options[index % color_options.len()].to_owned();
			BarChartSeries {
				color,
				data: vec![BarChartPoint {
					label: props.chart_data.label.to_owned(),
					x: 0.0,
					y: Some(entry.1.to_f64().unwrap()),
				}],
				title: Some(class.to_owned()),
			}
		})
		.collect::<Vec<_>>();
	html! {
		<BarChart
			id?="histogram_overall"
			series?={Some(series)}
			title?={Some(title)}
			y_min?={Some(0.0)}
		/>
	}
}

#[derive(Props)]
pub struct PredictionCountChartProps {
	pub chart_data: Vec<PredictionCountChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

#[component]
pub fn PredictionCountChart(props: PredictionCountChartProps) {
	let prediction_count_chart_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: props
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
		&props.date_window_interval,
		"Total Prediction Count".to_owned(),
	);
	html! {
		<BarChart
			id?="prediction_count"
			series?={Some(prediction_count_chart_series)}
			title?={Some(prediction_count_title)}
			y_min?={Some(0.0)}
		/>
	}
}

#[derive(Props)]
pub struct ColumnStatsTableProps {
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

#[component]
pub fn ColumnStatsTable(props: ColumnStatsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Status"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Column"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Absent Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Invalid Count"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
			{props.rows.into_iter().map(|row| html! {
				<ui::TableRow>
					<ui::TableCell>
						{if row.alert.is_some() {
							html! {
								<ui::AlertIcon
									alert={row.alert.unwrap()}
									level={ui::Level::Danger}
								>
									{"!"}
								</ui::AlertIcon>
							}
							} else {
								html! {
									<ui::AlertIcon
										alert="All good"
										level={ui::Level::Success}
									>
										{"✓"}
									</ui::AlertIcon>
								}
						}}
					</ui::TableCell>
					<ui::TableCell>
					{if row.href.is_some() {
						html! {
							<ui::Link href={row.href.unwrap()}>
								{row.name}
							</ui::Link>
						}
					} else {
						html! {<span>{row.name}</span>}
					}}
					</ui::TableCell>
					<ui::TableCell>
						{column_type_token(&row.column_type)}
					</ui::TableCell>
					<ui::TableCell>
						{row.absent_count.to_string()}
					</ui::TableCell>
					<ui::TableCell>
						{row.invalid_count.to_string()}
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct DateWindowSelectFormProps {
	pub date_window: DateWindow,
}

#[component]
pub fn DateWindowSelectForm(props: DateWindowSelectFormProps) {
	html! {
		<ui::Form>
			<DateWindowSelectField date_window={props.date_window} />
			<noscript>
				<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
					{"Submit"}
				</ui::Button>
			</noscript>
		</ui::Form>
	}
}
