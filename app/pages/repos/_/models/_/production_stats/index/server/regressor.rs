use crate::common::{
	ColumnStatsTable, ColumnStatsTableProps, DateWindowSelectForm, PredictionCountChart,
	PredictionCountChartEntry,
};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	time::{interval_chart_title, overall_chart_title},
};
use tangram_charts::{
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::BoxChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct RegressorProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: RegressorChartEntry,
	pub prediction_stats_interval_chart: Vec<RegressorChartEntry>,
	pub overall_column_stats_table_props: ColumnStatsTableProps,
}

#[component]
pub fn RegressorPage(props: RegressorProps) {
	html! {
		<ui::S1>
			<ui::H1>{"Production Stats"}</ui::H1>
			<DateWindowSelectForm date_window={props.date_window} />
			<ui::Card>
				<RegressionProductionStatsIntervalChart
					chart_data={props.prediction_stats_interval_chart}
					date_window_interval={props.date_window_interval}
				/>
			</ui::Card>
			<ui::Card>
				<PredictionCountChart
					chart_data={props.prediction_count_chart}
					date_window_interval={props.date_window_interval}
				/>
			</ui::Card>
			<ui::Card>
				<RegressionProductionStatsChart
					chart_data={props.prediction_stats_chart}
					date_window={props.date_window}
				/>
			</ui::Card>
			<ColumnStatsTable {props.overall_column_stats_table_props} />
		</ui::S1>
	}
}

#[derive(Props)]
pub struct RegressorChartsSectionProps {}

#[derive(Props)]
pub struct RegressionProductionStatsChartProps {
	pub chart_data: RegressorChartEntry,
	pub date_window: DateWindow,
}

pub struct RegressorChartEntry {
	pub label: String,
	pub quantiles: ProductionTrainingQuantiles,
}

pub struct ProductionTrainingQuantiles {
	pub production: Option<Quantiles>,
	pub training: Quantiles,
}

pub struct Quantiles {
	pub max: f32,
	pub min: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

#[component]
fn RegressionProductionStatsChart(props: RegressionProductionStatsChartProps) {
	let series = vec![
		BoxChartSeries {
			color: ui::colors::GREEN.to_owned(),
			data: vec![BoxChartPoint {
				label: props.chart_data.label.clone(),
				x: 0.0,
				y: Some(BoxChartValue {
					max: props.chart_data.quantiles.training.max.to_f64().unwrap(),
					min: props.chart_data.quantiles.training.min.to_f64().unwrap(),
					p25: props.chart_data.quantiles.training.p25.to_f64().unwrap(),
					p50: props.chart_data.quantiles.training.p50.to_f64().unwrap(),
					p75: props.chart_data.quantiles.training.p75.to_f64().unwrap(),
				}),
			}],
			title: Some("training quantiles".to_owned()),
		},
		BoxChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: vec![BoxChartPoint {
				label: props.chart_data.label,
				x: 0.0,
				y: props
					.chart_data
					.quantiles
					.production
					.map(|production| BoxChartValue {
						max: production.max.to_f64().unwrap(),
						min: production.min.to_f64().unwrap(),
						p25: production.p25.to_f64().unwrap(),
						p50: production.p50.to_f64().unwrap(),
						p75: production.p75.to_f64().unwrap(),
					}),
			}],
			title: Some("production quantiles".to_owned()),
		},
	];
	let title = overall_chart_title(
		&props.date_window,
		"Prediction Distribution Stats".to_owned(),
	);
	html! {
		<BoxChart
			id?="quantiles_overall"
			series?={Some(series)}
			title?={Some(title)}
		/>
	}
}

#[derive(Props)]
pub struct RegressionProductionStatsIntervalChartProps {
	pub chart_data: Vec<RegressorChartEntry>,
	pub date_window_interval: DateWindowInterval,
}

#[component]
fn RegressionProductionStatsIntervalChart(props: RegressionProductionStatsIntervalChartProps) {
	let series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: props
			.chart_data
			.into_iter()
			.enumerate()
			.map(|(index, entry)| BoxChartPoint {
				label: entry.label,
				x: index.to_f64().unwrap(),
				y: entry.quantiles.production.map(|production| BoxChartValue {
					max: production.max.to_f64().unwrap(),
					min: production.min.to_f64().unwrap(),
					p25: production.p25.to_f64().unwrap(),
					p50: production.p50.to_f64().unwrap(),
					p75: production.p75.to_f64().unwrap(),
				}),
			})
			.collect::<Vec<_>>(),
		title: Some("quantiles".to_owned()),
	}];
	let title = interval_chart_title(
		&props.date_window_interval,
		"Prediction Distribution Stats".to_owned(),
	);
	html! {
		<BoxChart
			id?="quantiles_intervals"
			series?={Some(series)}
			title?={Some(title)}
		/>
	}
}
