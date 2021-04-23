use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	class_select_field::ClassSelectField,
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
	metrics_row::MetricsRow,
	time::interval_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub model_layout_props: ModelLayoutProps,
	pub inner_props: InnerProps,
}

#[derive(Props)]
pub struct InnerProps {
	pub class_metrics: Vec<ClassMetricsEntry>,
	pub class: String,
	pub classes: Vec<String>,
	pub date_window_interval: DateWindowInterval,
	pub date_window: DateWindow,
	pub id: String,
	pub overall: OverallClassMetrics,
}

#[derive(Clone)]
pub struct ClassMetricsEntry {
	pub class_name: String,
	pub intervals: Vec<IntervalEntry>,
}

#[derive(Clone)]
pub struct IntervalEntry {
	pub label: String,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

#[derive(Clone)]
pub struct OverallClassMetrics {
	pub class_metrics: Vec<OverallClassMetricsEntry>,
	pub label: String,
}

#[derive(Clone)]
pub struct OverallClassMetricsEntry {
	pub class_name: String,
	pub confusion_matrix_training_production_comparison:
		ConfusionMatrixTrainingProductionComparison,
	pub confusion_matrix: ConfusionMatrix,
	pub training: Metrics,
	pub production: Option<Metrics>,
}

#[derive(Clone)]
pub struct Metrics {
	pub f1_score: f32,
	pub precision: f32,
	pub recall: f32,
}

#[derive(Clone)]
pub struct ConfusionMatrixTrainingProductionComparison {
	pub training: ConfusionMatrixFraction,
	pub production: Option<ConfusionMatrixFraction>,
}

#[derive(Clone)]
pub struct ConfusionMatrixFraction {
	pub false_negative_fraction: f32,
	pub true_negative_fraction: f32,
	pub true_positive_fraction: f32,
	pub false_positive_fraction: f32,
}

#[derive(Clone)]
pub struct ConfusionMatrix {
	pub false_negatives: Option<usize>,
	pub true_negatives: Option<usize>,
	pub true_positives: Option<usize>,
	pub false_positives: Option<usize>,
}

#[derive(Clone)]
pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<Inner {props.inner_props} />
			</ModelLayout>
		</Document>
	}
}

#[component]
pub fn Inner(props: InnerProps) {
	let selected_class_index = props
		.classes
		.iter()
		.position(|class| class == &props.class)
		.unwrap();
	let selected_class_interval_metrics = props.class_metrics[selected_class_index].clone();
	let selected_class_overall_metrics = props.overall.class_metrics[selected_class_index].clone();
	let intervals = selected_class_interval_metrics.intervals;
	let overall_training_metrics = selected_class_overall_metrics.training;
	let overall_production_metrics = selected_class_overall_metrics.production;
	html! {
		<ui::S1>
			<ui::H1>{"Production Metrics"}</ui::H1>
			<ui::TabBar>
				<ui::TabLink
					href=""
					selected={false}
				>
					{"Overview"}
				</ui::TabLink>
				<ui::TabLink
					href="class_metrics"
					selected={true}
				>
					{"Class Metrics"}
				</ui::TabLink>
			</ui::TabBar>
			<ui::Form>
				<DateWindowSelectField date_window={props.date_window} />
				<ClassSelectField class={props.class.clone()} classes={props.classes.clone()} />
				<noscript>
					<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
						{"Submit"}
					</ui::Button>
				</noscript>
			</ui::Form>
			<PrecisionRecallSection
				date_window_interval={props.date_window_interval}
				intervals={intervals}
				overall_training_metrics={overall_training_metrics}
				overall_production_metrics={overall_production_metrics}
			/>
			<ConfusionMatrixSection
				class={props.class.to_owned()}
				confusion_matrix={selected_class_overall_metrics.confusion_matrix}
			/>
			<ProductionTrainingSection
				class={props.class}
				confusion_matrix_training_production_comparison={selected_class_overall_metrics.confusion_matrix_training_production_comparison}
			/>
		</ui::S1>
	}
}

#[derive(Props)]
struct PrecisionRecallSectionProps {
	date_window_interval: DateWindowInterval,
	intervals: Vec<IntervalEntry>,
	overall_training_metrics: Metrics,
	overall_production_metrics: Option<Metrics>,
}

#[component]
fn PrecisionRecallSection(props: PrecisionRecallSectionProps) {
	let precision_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "Precision".to_owned());
	let recall_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "Recall".to_owned());
	let f1_score_interval_chart_title =
		interval_chart_title(&props.date_window_interval, "F1 Score".to_owned());
	let chart_labels = props
		.intervals
		.iter()
		.map(|interval| interval.label.clone())
		.collect::<Vec<_>>();

	let precision_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Finite::new(interval.precision.training.to_f64().unwrap()).ok(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Precision".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: interval
						.precision
						.production
						.and_then(|precision| Finite::new(precision.to_f64().unwrap()).ok()),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Precision".to_owned()),
		},
	];
	let recall_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Finite::new(interval.recall.training.to_f64().unwrap()).ok(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training Recall".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: interval
						.recall
						.production
						.and_then(|recall| Finite::new(recall.to_f64().unwrap()).ok()),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production Recall".to_owned()),
		},
	];
	let f1_score_chart_series = vec![
		LineChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Finite::new(interval.f1_score.training.to_f64().unwrap()).ok(),
				})
				.collect::<Vec<_>>(),
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Training F1 Score".to_owned()),
		},
		LineChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.intervals
				.iter()
				.enumerate()
				.map(|(index, interval)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: interval
						.f1_score
						.production
						.and_then(|f1_score| Finite::new(f1_score.to_f64().unwrap()).ok()),
				})
				.collect::<Vec<_>>(),
			line_style: None,
			point_style: None,
			title: Some("Production F1 Score".to_owned()),
		},
	];
	html! {
		<ui::S2>
			<ui::H2>{"Precision and Recall"}</ui::H2>
			<MetricsRow>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="Precision"
					value_a={Some(props.overall_training_metrics.precision)}
					value_a_title="Training"
					value_b={props.overall_production_metrics.as_ref().map(|value| value.precision)}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Percent(Default::default())}
				/>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="Recall"
					value_a={Some(props.overall_training_metrics.recall)}
					value_a_title="Training"
					value_b={props.overall_production_metrics.as_ref().map(|value| value.recall)}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Percent(Default::default())}
				/>
			</MetricsRow>
			<ui::Card>
				<LineChart
					id?="precision_intervals"
					labels?={Some(chart_labels.clone())}
					series?={Some(precision_chart_series)}
					title?={Some(precision_interval_chart_title)}
					x_axis_grid_line_interval?={Some(GridLineInterval { k: 1.0, p: 0.0 })}
					y_max?={Some(Finite::new(1.0).unwrap())}
					y_min?={Some(Finite::new(0.0).unwrap())}
				/>
			</ui::Card>
			<ui::Card>
				<LineChart
					id?="recall_intervals"
					x_axis_grid_line_interval?={
						Some(GridLineInterval { k: 1.0, p: 0.0 })
					}
					y_max?={Some(Finite::new(1.0).unwrap())}
					y_min?={Some(Finite::new(0.0).unwrap())}
					labels?={Some(chart_labels.clone())}
					series?={Some(recall_chart_series)}
					title?={Some(recall_interval_chart_title)}
				/>
			</ui::Card>
			<MetricsRow>
				<ui::NumberComparisonCard
					color_a={Some(TRAINING_COLOR.to_owned())}
					color_b={Some(PRODUCTION_COLOR.to_owned())}
					title="F1 Score"
					value_a={Some(props.overall_training_metrics.f1_score)}
					value_a_title="Training"
					value_b={props.overall_production_metrics.as_ref().map(|value| value.f1_score)}
					value_b_title="Production"
					number_formatter={ui::NumberFormatter::Float(Default::default())}
				/>
			</MetricsRow>
			<ui::Card>
				<LineChart
					id?="f1_intervals"
					x_axis_grid_line_interval?={
						Some(GridLineInterval { k: 1.0, p: 0.0 })
					}
					y_axis_grid_line_interval?={None}
					labels?={Some(chart_labels)}
					series?={Some(f1_score_chart_series)}
					title?={Some(f1_score_interval_chart_title)}
					y_min?={Some(Finite::new(0.0).unwrap())}
					y_max?={Some(Finite::new(1.0).unwrap())}
				/>
			</ui::Card>
		</ui::S2>
	}
}

#[derive(Props)]
struct ConfusionMatrixSectionProps {
	class: String,
	confusion_matrix: ConfusionMatrix,
}

#[component]
fn ConfusionMatrixSection(props: ConfusionMatrixSectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Confusion Matrix"}</ui::H2>
			<ui::ConfusionMatrix
				class_label={props.class.to_owned()}
				false_negatives={
					props.confusion_matrix.false_negatives
				}
				false_positives={
					props.confusion_matrix.false_positives
				}
				true_negatives={
					props.confusion_matrix.true_negatives
				}
				true_positives={
					props.confusion_matrix.true_positives
				}
			/>
		</ui::S2>
	}
}

#[derive(Props)]
struct ProductionTrainingSectionProps {
	class: String,
	confusion_matrix_training_production_comparison: ConfusionMatrixTrainingProductionComparison,
}

#[component]
fn ProductionTrainingSection(props: ProductionTrainingSectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Production v. Training Confusion Matrix"}</ui::H2>
			<ui::ConfusionMatrixComparison
				class_label={props.class.to_owned()}
				color_a={TRAINING_COLOR.to_owned()}
				color_b={PRODUCTION_COLOR.to_owned()}
				value_a={Some(ui::ConfusionMatrixComparisonValue {
					false_negative:
						props.confusion_matrix_training_production_comparison
							.training.false_negative_fraction,
					false_positive:
						props.confusion_matrix_training_production_comparison
							.training.false_positive_fraction,
					true_negative:
						props.confusion_matrix_training_production_comparison
							.training.true_negative_fraction,
					true_positive:
						props.confusion_matrix_training_production_comparison
							.training.true_positive_fraction,
				})}
				value_a_title="Training"
				value_b={props.confusion_matrix_training_production_comparison
							.production.as_ref().map(|production| ui::ConfusionMatrixComparisonValue {
					false_negative: production.false_negative_fraction,
					false_positive: production.false_positive_fraction,
					true_negative: production.true_negative_fraction,
					true_positive: production.true_positive_fraction,
				})}
				value_b_title="Production"
			/>
		</ui::S2>
	}
}
