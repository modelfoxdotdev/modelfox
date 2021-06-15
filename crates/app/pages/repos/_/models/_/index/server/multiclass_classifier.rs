use crate::common::{FeatureImportancesSection, TrainingSummarySection};
use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::colors::{BASELINE_COLOR, TRAINING_COLOR};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct MulticlassClassifier {
	pub id: String,
	pub warning: Option<String>,
	pub training_summary_section: TrainingSummarySection,
	pub training_metrics_section: MulticlassClassifierMetricsSection,
	pub feature_importances_section: FeatureImportancesSection,
}

impl Component for MulticlassClassifier {
	fn into_node(self) -> Node {
		ui::S1::new()
			.child(self.warning.map(|warning| {
				ui::Alert::new(ui::Level::Danger)
					.title("BAD MODEL".to_owned())
					.child(warning)
			}))
			.child(ui::H1::new().child("Overview"))
			.child(self.training_summary_section)
			.child(self.training_metrics_section)
			.child(self.feature_importances_section)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct MulticlassClassifierMetricsSection {
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

impl Component for MulticlassClassifierMetricsSection {
	fn into_node(self) -> Node {
		let losses_chart_series = self.losses_chart_series.map(|losses_chart_series| {
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
		let title = ui::H2::new().child("Metrics");
		let p = ui::P::new()
			.child("Your model was evaluated on the test dataset and accurately classified ")
			.child(b().child(ui::format_percent(self.accuracy)))
			.child(" of the examples. This is compared with the baseline accuracy of ")
			.child(b().child(ui::format_percent(self.baseline_accuracy)))
			.child(
				", which is what the model would get if it always predicted the majority class.",
			);
		ui::S2::new()
			.child(title)
			.child(p)
			.child(
				ui::NumberComparisonCard::new(Some(self.baseline_accuracy), Some(self.accuracy))
					.color_a(Some(BASELINE_COLOR.to_owned()))
					.color_b(Some(TRAINING_COLOR.to_owned()))
					.title("Accuracy".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Training".to_owned())
					.number_formatter(ui::NumberFormatter::Percent(Default::default())),
			)
			.child(losses_chart_series.map(|losses_chart_series| {
				ui::Card::new().child(Dehydrate::new(
					"loss",
					LineChart::new()
						.series(Some(losses_chart_series))
						.title("Training Loss By Round or Epoch".to_owned())
						.x_axis_title("Round or Epoch".to_owned())
						.y_axis_title("Loss".to_owned())
						.y_min(Some(Finite::new(0.0).unwrap())),
				))
			}))
			.into_node()
	}
}
