use crate::common::{FeatureImportancesSection, TrainingSummarySection};
use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::metrics_row::MetricsRow;
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct BinaryClassifier {
	pub id: String,
	pub warning: Option<String>,
	pub training_summary_section: TrainingSummarySection,
	pub training_metrics_section: BinaryClassifierMetricsSection,
	pub feature_importances_section: FeatureImportancesSection,
}

impl Component for BinaryClassifier {
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
pub struct BinaryClassifierMetricsSection {
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub losses_chart_series: Option<Vec<f32>>,
}

impl Component for BinaryClassifierMetricsSection {
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
						y: Finite::new(loss.to_f64().unwrap()).ok(),
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
		let auc_roc = ui::NumberCard::new("AUC ROC".to_owned(), ui::format_percent(self.auc_roc));
		let accuracy =
			ui::NumberCard::new("Accuracy".to_owned(), ui::format_percent(self.accuracy));
		let pr = MetricsRow::new()
			.child(ui::NumberCard::new(
				"Precision".to_owned(),
				ui::format_percent(self.precision),
			))
			.child(ui::NumberCard::new(
				"Recall".to_owned(),
				ui::format_percent(self.recall),
			));
		let losses_chart = losses_chart_series.map(|losses_chart_series| {
			ui::Card::new().child(
				LineChart::new()
					.id("loss".to_owned())
					.series(Some(losses_chart_series))
					.title("Training Loss By Round or Epoch".to_owned())
					.x_axis_title("Round or Epoch".to_owned())
					.y_axis_title("Loss".to_owned())
					.y_min(Some(Finite::new(0.0).unwrap())),
			)
		});
		ui::S2::new()
			.child(title)
			.child(p)
			.child(auc_roc)
			.child(accuracy)
			.child(pr)
			.child(losses_chart)
			.into_node()
	}
}
