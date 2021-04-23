use crate::page::{TrainingProductionMetrics, TrueValuesCountChartEntry};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	metrics_row::MetricsRow,
	time::interval_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(Props)]
pub struct RegressorProductionMetricsProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub mse_chart: MeanSquaredErrorChart,
	pub overall: RegressionProductionMetrics,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
}

pub struct MeanSquaredErrorChart {
	pub data: Vec<MeanSquaredErrorChartEntry>,
	pub training_mse: f32,
}

pub struct MeanSquaredErrorChartEntry {
	pub label: String,
	pub mse: Option<f32>,
}

pub struct RegressionProductionMetrics {
	pub mse: TrainingProductionMetrics,
	pub rmse: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[component]
pub fn RegressorProductionMetrics(props: RegressorProductionMetricsProps) {
	let mse_chart_labels = props
		.mse_chart
		.data
		.iter()
		.map(|entry| entry.label.clone())
		.collect::<Vec<_>>();
	let mse_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: (0..props.mse_chart.data.len())
				.map(|index| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Some(Finite::new(props.mse_chart.training_mse.to_f64().unwrap()).unwrap()),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Mean Squared Error".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.mse_chart
				.data
				.iter()
				.enumerate()
				.map(|(index, entry)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: entry
						.mse
						.map(|mse| Finite::new(mse.to_f64().unwrap()).unwrap()),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Mean Squared Error".to_owned()),
		},
	];
	let mse_chart_title =
		interval_chart_title(&props.date_window_interval, "Mean Squared Error".to_owned());
	html! {
	<ui::S1>
		<ui::H1>{"Production Metrics"}</ui::H1>
		<ui::S2>
			<ui::Form>
				<DateWindowSelectField date_window={props.date_window} />
				<noscript>
					<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
						{"Submit"}
					</ui::Button>
				</noscript>
			</ui::Form>
			<ui::P>
				{"You have logged "}
				<b>{props.overall.true_values_count.to_string()}</b>
				{" true values for this date range."}
			</ui::P>
			<ui::Card>
				<LineChart
					id?="mse"
					labels?={Some(mse_chart_labels)}
					series?={Some(mse_series)}
					title?={Some(mse_chart_title)}
					x_axis_grid_line_interval?={
						Some(GridLineInterval { k: 1.0, p: 0.0 })
					}
					y_max?={Some(Finite::new(1.0).unwrap())}
					y_min?={Some(Finite::new(0.0).unwrap())}
				/>
			</ui::Card>
			<MetricsRow>
				<ui::NumberCard
					title="True Value Count"
					value={props.overall.true_values_count.to_string()}
				/>
			</MetricsRow>
			<MetricsRow>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="Root Mean Squared Error"
					value_a={Some(props.overall.rmse.training)}
					value_a_title="Training"
					value_b={props.overall.rmse.production}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Float(Default::default())}
				/>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="Mean Squared Error"
					value_a={Some(props.overall.mse.training)}
					value_a_title="Training"
					value_b={props.overall.mse.production}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Float(Default::default())}
				/>
			</MetricsRow>
		</ui::S2>
	</ui::S1>
	}
}
