use crate::common::{
	FeatureImportancesSection, FeatureImportancesSectionProps, TrainingSummarySection,
	TrainingSummarySectionProps,
};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(Props)]
pub struct BinaryClassifierProps {
	pub id: String,
	pub warning: Option<String>,
	pub training_summary_section_props: TrainingSummarySectionProps,
	pub training_metrics_section_props: BinaryClassifierMetricsSectionProps,
	pub feature_importances_section_props: FeatureImportancesSectionProps,
}

pub fn binary_classifier_index_page(props: BinaryClassifierProps) -> html::Node {
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
			<TrainingSummarySection {props.training_summary_section_props} />
			<TrainingMetricsSection {props.training_metrics_section_props} />
			<FeatureImportancesSection {props.feature_importances_section_props} />
		</ui::S1>
	}
}

#[derive(Props)]
pub struct BinaryClassifierMetricsSectionProps {
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub losses_chart_series: Option<Vec<f32>>,
}

#[component]
fn TrainingMetricsSection(props: BinaryClassifierMetricsSectionProps) {
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
					y: Finite::new(loss.to_f64().unwrap()).ok(),
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
			<ui::NumberCard
				title="AUC ROC"
				value={ui::format_percent(props.auc_roc)}
			/>
			<ui::NumberCard
				title="Accuracy"
				value={ui::format_percent(props.accuracy)}
			/>
			<MetricsRow>
				<ui::NumberCard
					title="Precision"
					value={ui::format_percent(props.precision)}
				/>
				<ui::NumberCard
					title="Recall"
					value={ui::format_percent(props.recall)}
				/>
			</MetricsRow>
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
