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
	pub id: String,
	pub roc_curve_data: Vec<RocCurveData>,
	pub model_layout_info: ModelLayoutInfo,
	pub class: String,
	pub auc_roc: f32,
}

pub struct RocCurveData {
	pub false_positive_rate: f32,
	pub true_positive_rate: f32,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let aucroc_description = "The area under the receiver operating characteric curve is the probability that a randomly chosen positive example's predicted score is higher than a randomly selected negative example's score. A value of 100% means your model is perfectly able to classify positive and negative rows. A value of 50% means your model is unable to distinguish positive rows from negative rows. A value of 0% means your model is perfectly mis-classifying positive rows as negative and negative rows as positive.";
		let roc_description = "The Receiver Operating Characteristic Curve shows the True Positive Rate v. False Positive Rate at various thresholds.";
		let roc_series = self
			.roc_curve_data
			.iter()
			.map(|roc_curve_series| LineChartPoint {
				x: Finite::new(roc_curve_series.false_positive_rate.to_f64().unwrap()).unwrap(),
				y: Finite::new(roc_curve_series.true_positive_rate.to_f64().unwrap()).ok(),
			})
			.collect::<Vec<_>>();
		let roc_series = vec![
			LineChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: roc_series,
				line_style: Some(LineStyle::Solid),
				point_style: Some(PointStyle::Circle),
				title: Some("ROC".to_owned()),
			},
			LineChartSeries {
				color: ui::colors::GRAY.to_owned(),
				data: vec![
					LineChartPoint {
						x: Finite::new(0.0).unwrap(),
						y: Finite::new(0.0).ok(),
					},
					LineChartPoint {
						x: Finite::new(1.0).unwrap(),
						y: Finite::new(1.0).ok(),
					},
				],
				line_style: Some(LineStyle::Dashed),
				point_style: Some(PointStyle::Hidden),
				title: Some("Reference".to_owned()),
			},
		];
		let content = ui::S1::new()
			.child(ui::H1::new().child("Training Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("./".to_owned(), false).child("Overview"))
					.child(ui::TabLink::new("precision_recall".to_owned(), false).child("PR Curve"))
					.child(ui::TabLink::new("roc".to_owned(), true).child("ROC Curve")),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Area Under the Receiver Operating Characteristic"))
					.child(ui::P::new().child(aucroc_description))
					.child(ui::NumberCard::new(
						"AUC ROC".to_owned(),
						ui::format_percent(self.auc_roc),
					)),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Receiver Operating Characteristic Curve"))
					.child(ui::P::new().child(roc_description))
					.child(
						ui::Card::new().child(
							LineChart::new()
								.id("roc".to_owned())
								.series(Some(roc_series))
								.title("Receiver Operating Characteristic Curve".to_owned())
								.x_axis_title("False Positive Rate".to_owned())
								.y_axis_title("True Positive Rate".to_owned())
								.x_max(Some(Finite::new(1.0).unwrap()))
								.x_min(Some(Finite::new(0.0).unwrap()))
								.y_max(Some(Finite::new(1.0).unwrap()))
								.y_min(Some(Finite::new(0.0).unwrap())),
						),
					),
			);
		Document::new()
			.client("tangram_app_training_metrics_roc_client")
			.child(ModelLayout::new(self.model_layout_info).child(content))
			.into_node()
	}
}
