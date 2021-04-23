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
pub struct RegressorProps {
	pub id: String,
	pub warning: Option<String>,
	pub metrics_section_props: RegressorMetricsSectionProps,
	pub summary_section_props: TrainingSummarySectionProps,
	pub feature_importances_section_props: FeatureImportancesSectionProps,
}

pub fn regressor_index_page(props: RegressorProps) -> html::Node {
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
pub struct RegressorMetricsSectionProps {
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
	pub losses_chart_series: Option<Vec<f32>>,
}

#[component]
fn TrainingMetricsSection(props: RegressorMetricsSectionProps) {
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
				{"Your model was evaluated on the test dataset and achieved a root mean squared error of "}
				<b>{ui::format_float(props.rmse)}</b>
				{". This is compared with the baseline root mean squared error of "}
				<b>{ui::format_float(props.baseline_rmse)}</b>
				{", which is what the model would get if it always predicted the mean."}
			</ui::P>
			<ui::NumberComparisonCard
				color_a={Some(BASELINE_COLOR.to_owned())}
				color_b={Some(TRAINING_COLOR.to_owned())}
				title="Root Mean Squared Error"
				value_a={Some(props.baseline_rmse)}
				value_a_title="Baseline"
				value_b={Some(props.rmse)}
				value_b_title="Training"
				number_formatter={ui::NumberFormatter::Float(Default::default())}
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
