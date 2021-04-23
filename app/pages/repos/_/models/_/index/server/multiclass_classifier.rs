use crate::common::{
	FeatureImportancesSection, FeatureImportancesSectionProps, TrainingSummarySection,
	TrainingSummarySectionProps,
};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::tokens::{BASELINE_COLOR, TRAINING_COLOR};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(Props)]
pub struct MulticlassClassifierProps {
	pub id: String,
	pub warning: Option<String>,
	pub summary_section_props: TrainingSummarySectionProps,
	pub metrics_section_props: MulticlassClassifierMetricsSectionProps,
	pub feature_importances_section_props: FeatureImportancesSectionProps,
}

pub fn multiclass_classifier_index_page(props: MulticlassClassifierProps) -> html::Node {
	html! {
		<ui::S1>
			{props.warning.map(|warning| {
				html! {
					<ui::Alert level={ui::Level::Danger} title?="BAD MODEL">
						{warning}
					</ui::Alert>
				}
			})}
			<ui::H1>{"Overview"}</ui::H1>
			<TrainingSummarySection {props.summary_section_props} />
			<TrainingMetricsSection {props.metrics_section_props} />
			<FeatureImportancesSection {props.feature_importances_section_props} />
		</ui::S1>
	}
}

#[derive(Props)]
pub struct MulticlassClassifierMetricsSectionProps {
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<MulticlassClassifierClassMetrics>,
	pub classes: Vec<String>,
	pub losses_chart_series: Option<Vec<f32>>,
}

pub struct MulticlassClassifierClassMetrics {
	pub precision: f32,
	pub recall: f32,
}

#[component]
fn TrainingMetricsSection(props: MulticlassClassifierMetricsSectionProps) {
	let losses_chart_series = props.losses_chart_series.map(|losses_chart_series| {
		vec![LineChartSeries {
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			color: ui::colors::BLUE.to_string(),
			data: losses_chart_series
				.iter()
				.enumerate()
				.map(|(index, loss)| LineChartPoint {
					x: Finite::new(index.to_f64().unwrap()).unwrap(),
					y: Some(Finite::new(loss.to_f64().unwrap()).unwrap()),
				})
				.collect::<Vec<_>>(),
			title: Some("loss".to_owned()),
		}]
	});
	html! {
		<ui::S2>
			<ui::H2>{"Metrics"}</ui::H2>
			<ui::P>
				{"Your model was evaluated on the test dataset and accurately classified "}
				<b>{ui::format_percent(props.accuracy)}</b>
				{" of the examples. This is compared with the baseline accuracy of "}
				<b>{ui::format_percent(props.baseline_accuracy)}</b>
				{", which is what the model would get if it always predicted the majority class."}
			</ui::P>
			<ui::NumberComparisonCard
				color_a={Some(BASELINE_COLOR.to_owned())}
				color_b={Some(TRAINING_COLOR.to_owned())}
				title="Accuracy"
				value_a={Some(props.baseline_accuracy)}
				value_a_title="Baseline"
				value_b={Some(props.accuracy)}
				value_b_title="Training"
				number_formatter={ui::NumberFormatter::Percent(Default::default())}
			/>
			{losses_chart_series.map(|losses_chart_series| html! {
				<ui::Card>
					<LineChart
						id?="loss"
						series?={Some(losses_chart_series)}
						title?="Training Loss By Round or Epoch"
						x_axis_title?="Round or Epoch"
						y_axis_title?="Loss"
						y_min?={Some(Finite::new(0.0).unwrap())}
					/>
				</ui::Card>
			})}
		</ui::S2>
	}
}
