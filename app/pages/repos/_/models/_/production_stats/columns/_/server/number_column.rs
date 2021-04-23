use crate::page::{IntervalBoxChartDataPoint, OverallBoxChartData};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	metrics_row::MetricsRow,
	time::{interval_chart_title, overall_chart_title},
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue};
use tangram_charts::components::BoxChart;
use tangram_ui as ui;

#[derive(Props)]
pub struct NumberColumnProps {
	pub column_name: String,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub alert: Option<String>,
	pub number_column_counts_section_props: NumberColumnCountsSectionProps,
	pub interval_box_chart_data: Vec<IntervalBoxChartDataPoint>,
	pub overall_box_chart_data: OverallBoxChartData,
	pub number_column_stats_section_props: NumberColumnStatsSectionProps,
}

pub struct NumberTrainingProductionComparison {
	pub production: Option<f32>,
	pub training: f32,
}

#[component]
pub fn NumberColumn(props: NumberColumnProps) {
	let stats_overall_chart_title = overall_chart_title(&props.date_window, "Stats".to_owned());
	let overall_box_chart_series = vec![
		BoxChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: vec![BoxChartPoint {
				label: "Training".to_owned(),
				x: 0.0,
				y: Some(BoxChartValue {
					max: props.overall_box_chart_data.training.max.to_f64().unwrap(),
					min: props.overall_box_chart_data.training.min.to_f64().unwrap(),
					p25: props.overall_box_chart_data.training.p25.to_f64().unwrap(),
					p50: props.overall_box_chart_data.training.p50.to_f64().unwrap(),
					p75: props.overall_box_chart_data.training.p75.to_f64().unwrap(),
				}),
			}],
			title: Some(format!("Training Stats for {}", props.column_name)),
		},
		BoxChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: vec![BoxChartPoint {
				label: "Production".to_owned(),
				x: 0.0,
				y: props
					.overall_box_chart_data
					.production
					.map(|production| BoxChartValue {
						max: production.max.to_f64().unwrap(),
						min: production.min.to_f64().unwrap(),
						p25: production.p25.to_f64().unwrap(),
						p50: production.p50.to_f64().unwrap(),
						p75: production.p75.to_f64().unwrap(),
					}),
			}],
			title: Some(format!("Production Stats for {}", props.column_name)),
		},
	];
	let stats_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "Stats".to_owned());
	let interval_box_chart_series = vec![BoxChartSeries {
		color: PRODUCTION_COLOR.to_owned(),
		data: props
			.interval_box_chart_data
			.iter()
			.enumerate()
			.map(|(index, entry)| BoxChartPoint {
				label: entry.label.to_owned(),
				x: index.to_f64().unwrap(),
				y: entry.stats.as_ref().map(|stats| BoxChartValue {
					max: stats.max.to_f64().unwrap(),
					min: stats.min.to_f64().unwrap(),
					p25: stats.p25.to_f64().unwrap(),
					p50: stats.p50.to_f64().unwrap(),
					p75: stats.p75.to_f64().unwrap(),
				}),
			})
			.collect(),
		title: Some(format!("Production Stats for {}", props.column_name)),
	}];
	html! {
		<>
			{props.alert.map(|alert| html! {
				<ui::Alert level={ui::Level::Danger}>
					{alert}
				</ui::Alert>
			})}
			<NumberColumnCountsSection {props.number_column_counts_section_props} />
			<ui::Card>
				<BoxChart
					id?="number_overall"
					series?={Some(overall_box_chart_series)}
					title?={Some(stats_overall_chart_title)}
				/>
			</ui::Card>
			<ui::Card>
				<BoxChart
					id?="number_intervals"
					series?={Some(interval_box_chart_series)}
					title?={Some(stats_interval_chart_title)}
				/>
			</ui::Card>
			<NumberColumnStatsSection {props.number_column_stats_section_props} />
		</>
	}
}

#[derive(Props)]
pub struct NumberColumnCountsSectionProps {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

#[component]
fn NumberColumnCountsSection(props: NumberColumnCountsSectionProps) {
	html! {
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
	}
}

#[derive(Props)]
pub struct NumberColumnStatsSectionProps {
	pub max_comparison: NumberTrainingProductionComparison,
	pub mean_comparison: NumberTrainingProductionComparison,
	pub min_comparison: NumberTrainingProductionComparison,
	pub std_comparison: NumberTrainingProductionComparison,
}

#[component]
fn NumberColumnStatsSection(props: NumberColumnStatsSectionProps) {
	html! {
		<>
			<ui::S2>
				<MetricsRow>
					<ui::NumberComparisonCard
						color_a={Some(TRAINING_COLOR.to_owned())}
						color_b={Some(PRODUCTION_COLOR.to_owned())}
						title="Min"
						value_a={Some(props.min_comparison.training)}
						value_a_title="Training"
						value_b={props.min_comparison.production}
						value_b_title="Production"
						number_formatter={ui::NumberFormatter::Float(Default::default())}
					/>
					<ui::NumberComparisonCard
						color_a={Some(TRAINING_COLOR.to_owned())}
						color_b={Some(PRODUCTION_COLOR.to_owned())}
						title="Max"
						value_a={Some(props.max_comparison.training)}
						value_a_title="Training"
						value_b={props.max_comparison.production}
						value_b_title="Production"
						number_formatter={ui::NumberFormatter::Float(Default::default())}
					/>
				</MetricsRow>
				<MetricsRow>
					<ui::NumberComparisonCard
						color_a={Some(TRAINING_COLOR.to_owned())}
						color_b={Some(PRODUCTION_COLOR.to_owned())}
						title="Mean"
						value_a={Some(props.mean_comparison.training)}
						value_a_title="Training"
						value_b={props.mean_comparison.production}
						value_b_title="Production"
						number_formatter={ui::NumberFormatter::Float(Default::default())}
					/>
					<ui::NumberComparisonCard
						color_a={Some(TRAINING_COLOR.to_owned())}
						color_b={Some(PRODUCTION_COLOR.to_owned())}
						title="Standard Deviation"
						value_a={Some(props.std_comparison.training)}
						value_a_title="Training"
						value_b={props.std_comparison.production}
						value_b_title="Production"
						number_formatter={ui::NumberFormatter::Float(Default::default())}
					/>
				</MetricsRow>
			</ui::S2>
		</>
	}
}
