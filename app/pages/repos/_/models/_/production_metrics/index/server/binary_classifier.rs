use crate::page::{AccuracyChart, TrainingProductionMetrics, TrueValuesCountChartEntry};
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
pub struct BinaryClassifierProductionMetricsProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub true_values_count_chart: Vec<TrueValuesCountChartEntry>,
	pub overall: BinaryClassificationOverallProductionMetrics,
	pub id: String,
	pub accuracy_chart: AccuracyChart,
}

pub struct BinaryClassificationOverallProductionMetrics {
	pub accuracy: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
	pub true_values_count: u64,
}

#[component]
pub fn BinaryClassifierProductionMetrics(props: BinaryClassifierProductionMetricsProps) {
	let chart_labels = props
		.accuracy_chart
		.data
		.iter()
		.map(|entry| entry.label.clone())
		.collect::<Vec<_>>();
	let accuracy_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: (0..props.accuracy_chart.data.len())
				.map(|index| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Some(
						Finite::new(props.accuracy_chart.training_accuracy.to_f64().unwrap())
							.unwrap(),
					),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Accuracy".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.accuracy_chart
				.data
				.iter()
				.enumerate()
				.map(|(index, entry)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: entry
						.accuracy
						.map(|accuracy| Finite::new(accuracy.to_f64().unwrap()).unwrap()),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Accuracy".to_owned()),
		},
	];
	let accuracy_chart_title =
		interval_chart_title(&props.date_window_interval, "Accuracy".to_owned());
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
				<MetricsRow>
					<ui::NumberCard
						title="True Value Count"
						value={props.overall.true_values_count.to_string()}
					/>
				</MetricsRow>
			</ui::S2>
			<ui::S2>
				<ui::H2>{"Accuracy"}</ui::H2>
				<ui::P>{"Accuracy is the percentage of predictions that were correct."}</ui::P>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="Accuracy"
					value_a={Some(props.overall.accuracy.training)}
					value_a_title="Training"
					value_b={props.overall.accuracy.production}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Percent(Default::default())}
				/>
				<ui::Card>
					<LineChart
						id?="accuracy"
						labels?={Some(chart_labels)}
						series?={Some(accuracy_series)}
						title?={Some(accuracy_chart_title)}
						x_axis_grid_line_interval?={
							Some(GridLineInterval { k: 1.0, p: 0.0 })
						}
						y_max?={Some(Finite::new(1.0).unwrap())}
						y_min?={Some(Finite::new(0.0).unwrap())}
					/>
				</ui::Card>
			</ui::S2>
		</ui::S1>
	}
}
