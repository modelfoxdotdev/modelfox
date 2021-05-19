use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub class: String,
	pub precision_recall_curve_series: Vec<PrecisionRecallPoint>,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
}

pub struct PrecisionRecallPoint {
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let pr_series = self
			.precision_recall_curve_series
			.iter()
			.map(|threshold| LineChartPoint {
				x: Finite::new(threshold.recall.to_f64().unwrap()).unwrap(),
				y: Finite::new(threshold.precision.to_f64().unwrap()).ok(),
			})
			.collect::<Vec<_>>();
		let precision_series = self
			.precision_recall_curve_series
			.iter()
			.map(|threshold| LineChartPoint {
				x: Finite::new(threshold.threshold.to_f64().unwrap()).unwrap(),
				y: Finite::new(threshold.precision.to_f64().unwrap()).ok(),
			})
			.collect::<Vec<_>>();
		let recall_series = self
			.precision_recall_curve_series
			.iter()
			.map(|threshold| LineChartPoint {
				x: Finite::new(threshold.threshold.to_f64().unwrap()).unwrap(),
				y: Finite::new(threshold.recall.to_f64().unwrap()).ok(),
			})
			.collect::<Vec<_>>();
		let parametric_series = vec![LineChartSeries {
			line_style: Some(LineStyle::Solid),
			point_style: Some(PointStyle::Circle),
			color: ui::colors::BLUE.to_owned(),
			data: pr_series,
			title: Some("PR".to_owned()),
		}];
		let non_parametric_series = vec![
			LineChartSeries {
				line_style: Some(LineStyle::Solid),
				point_style: Some(PointStyle::Circle),
				color: ui::colors::BLUE.to_owned(),
				data: precision_series,
				title: Some("Precision".to_owned()),
			},
			LineChartSeries {
				line_style: Some(LineStyle::Solid),
				point_style: Some(PointStyle::Circle),
				color: ui::colors::GREEN.to_owned(),
				data: recall_series,
				title: Some("Recall".to_owned()),
			},
		];
		let parametric_pr_curve_definition  = "The parametric precision recall curve shows the value of precision on the y axis for each value of recall on the x axis where each point is at a distinct threshold.";
		let non_parametric_pr_curve_definition = "The non-parametric precision recall curve shows the value of precision and recall the model would get on the y axis for each threshold on the x axis.";
		let content = ui::S1::new()
			.child(ui::H1::new().child("Training Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("./".to_owned(), false).child("Overview"))
					.child(ui::TabLink::new("precision_recall".to_owned(), true).child("PR Curve"))
					.child(ui::TabLink::new("roc".to_owned(), false).child("ROC Curve")),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Parametric Precision Recall Curve"))
					.child(ui::P::new().child(parametric_pr_curve_definition))
					.child(
						ui::Card::new().child(
							LineChart::new()
								.id("parametric_pr".to_owned())
								.hide_legend(Some(true))
								.series(Some(parametric_series))
								.title("Parametric Precision Recall Curve".to_owned())
								.x_axis_title("Recall".to_owned())
								.y_axis_title("Precision".to_owned())
								.x_max(Some(Finite::new(1.0).unwrap()))
								.x_min(Some(Finite::new(0.0).unwrap()))
								.y_max(Some(Finite::new(1.0).unwrap()))
								.y_min(Some(Finite::new(0.0).unwrap())),
						),
					),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Non-Parametric Precision Recall Curve"))
					.child(ui::P::new().child(non_parametric_pr_curve_definition))
					.child(
						ui::Card::new().child(
							LineChart::new()
								.id("non_parametric_pr".to_owned())
								.hide_legend(Some(true))
								.series(Some(non_parametric_series))
								.title("Non-Parametric Precision Recall Curve".to_owned())
								.x_axis_title("Threshold".to_owned())
								.y_axis_title("Precision/Recall".to_owned())
								.x_max(Some(Finite::new(1.0).unwrap()))
								.x_min(Some(Finite::new(0.0).unwrap()))
								.y_max(Some(Finite::new(1.0).unwrap()))
								.y_min(Some(Finite::new(0.0).unwrap())),
						),
					),
			);
		Document::new()
			.client("tangram_app_training_metrics_precision_recall_client")
			.child(ModelLayout::new(self.model_layout_info).child(content))
			.into_node()
	}
}
