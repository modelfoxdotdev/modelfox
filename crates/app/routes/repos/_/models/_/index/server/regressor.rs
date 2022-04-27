use crate::common::{FeatureImportancesSection, TrainingSummarySection};
use modelfox_app_ui::colors::{BASELINE_COLOR, TRAINING_COLOR};
use modelfox_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use modelfox_finite::Finite;
use modelfox_ui as ui;
use num::ToPrimitive;
use pinwheel::prelude::*;

pub struct Regressor {
	pub id: String,
	pub warning: Option<String>,
	pub training_summary_section: TrainingSummarySection,
	pub training_metrics_section: RegressorMetricsSection,
	pub feature_importances_section: Option<FeatureImportancesSection>,
}

impl Component for Regressor {
	fn into_node(self) -> Node {
		ui::S1::new()
			.child(self.warning.map(|warning| {
				ui::Alert::new(ui::Level::Danger)
					.title("BAD MODEL".to_owned())
					.child(warning)
			}))
			.child(ui::H1::new("Overview"))
			.child(self.training_summary_section)
			.child(self.training_metrics_section)
			.child(self.feature_importances_section)
			.into_node()
	}
}

pub struct RegressorMetricsSection {
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
	pub losses_chart_series: Option<Vec<f32>>,
}

impl Component for RegressorMetricsSection {
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
		let title = ui::H2::new("Metrics");
		let p = ui::P::new()
			.child("Your model was evaluated on the test dataset and achieved a root mean squared error of ")
			.child(b().child(ui::format_float(self.rmse)))
			.child(". This is compared with the baseline root mean squared error of ")
			.child(b().child(ui::format_float(self.baseline_rmse)))
			.child(", which is what the model would get if it always predicted the mean.");
		ui::S2::new()
			.child(title)
			.child(p)
			.child(
				ui::NumberComparisonCard::new(Some(self.baseline_rmse), Some(self.rmse))
					.color_a(BASELINE_COLOR.to_owned())
					.color_b(TRAINING_COLOR.to_owned())
					.title("Root Mean Squared Error".to_owned())
					.value_a_title("Baseline".to_owned())
					.value_b_title("Training".to_owned())
					.number_formatter(ui::NumberFormatter::Float(Default::default())),
			)
			.child(losses_chart_series.map(|losses_chart_series| {
				ui::Card::new().child(Dehydrate::new(
					"loss",
					LineChart::new()
						.series(losses_chart_series)
						.title("Training Loss By Round or Epoch".to_owned())
						.x_axis_title("Round or Epoch".to_owned())
						.y_axis_title("Loss".to_owned())
						.y_min(Finite::new(0.0).unwrap()),
				))
			}))
			.into_node()
	}
}
