use pinwheel::prelude::*;
use tangram_charts::{
	common::GridLineInterval,
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct ProductionMetrics {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ProductionMetrics {
	fn into_node(self) -> Node {
		let month_labels = vec![
			"Jan 2020".to_owned(),
			"Feb 2020".to_owned(),
			"Mar 2020".to_owned(),
			"Apr 2020".to_owned(),
			"May 2020".to_owned(),
			"Jun 2020".to_owned(),
			"Jul 2020".to_owned(),
			"Aug 2020".to_owned(),
			"Sep 2020".to_owned(),
			"Oct 2020".to_owned(),
			"Nov 2020".to_owned(),
			"Dec 2020".to_owned(),
		];
		let accuracy_data = vec![
			LineChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: vec![
					LineChartPoint {
						x: Finite::new(0.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(1.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(2.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(3.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(4.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(5.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(6.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(7.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(8.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(9.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(10.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(11.0).unwrap(),
						y: Some(Finite::new(0.8360655903816223).unwrap()),
					},
				],
				line_style: Some(LineStyle::Solid),
				point_style: Some(PointStyle::Circle),
				title: Some("Training".to_owned()),
			},
			LineChartSeries {
				color: ui::colors::GREEN.to_owned(),
				data: vec![
					LineChartPoint {
						x: Finite::new(0.0).unwrap(),
						y: Some(Finite::new(0.827037).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(1.0).unwrap(),
						y: Some(Finite::new(0.83504676).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(2.0).unwrap(),
						y: Some(Finite::new(0.80508476).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(3.0).unwrap(),
						y: Some(Finite::new(0.6696226).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(4.0).unwrap(),
						y: Some(Finite::new(0.79173913).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(5.0).unwrap(),
						y: Some(Finite::new(0.77857144).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(6.0).unwrap(),
						y: Some(Finite::new(0.78812322).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(7.0).unwrap(),
						y: Some(Finite::new(0.80912182).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(8.0).unwrap(),
						y: Some(Finite::new(0.81312818).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(9.0).unwrap(),
						y: Some(Finite::new(0.81283228).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(10.0).unwrap(),
						y: Some(Finite::new(0.80182129).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(11.0).unwrap(),
						y: Some(Finite::new(0.81112314).unwrap()),
					},
				],
				line_style: None,
				point_style: None,
				title: Some("Production".to_owned()),
			},
		];
		let precision_data = vec![
			LineChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: vec![
					LineChartPoint {
						x: Finite::new(0.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(1.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(2.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(3.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(4.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(5.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(6.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(7.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(8.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(9.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(10.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(11.0).unwrap(),
						y: Some(Finite::new(0.803).unwrap()),
					},
				],
				line_style: Some(LineStyle::Solid),
				point_style: Some(PointStyle::Circle),
				title: Some("Training".to_owned()),
			},
			LineChartSeries {
				color: ui::colors::GREEN.to_owned(),
				data: vec![
					LineChartPoint {
						x: Finite::new(0.0).unwrap(),
						y: Some(Finite::new(0.807037).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(1.0).unwrap(),
						y: Some(Finite::new(0.80204676).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(2.0).unwrap(),
						y: Some(Finite::new(0.80108476).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(3.0).unwrap(),
						y: Some(Finite::new(0.7296226).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(4.0).unwrap(),
						y: Some(Finite::new(0.74173913).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(5.0).unwrap(),
						y: Some(Finite::new(0.78857144).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(6.0).unwrap(),
						y: Some(Finite::new(0.79812322).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(7.0).unwrap(),
						y: Some(Finite::new(0.76912182).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(8.0).unwrap(),
						y: Some(Finite::new(0.77312818).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(9.0).unwrap(),
						y: Some(Finite::new(0.80283228).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(10.0).unwrap(),
						y: Some(Finite::new(0.80182129).unwrap()),
					},
					LineChartPoint {
						x: Finite::new(11.0).unwrap(),
						y: Some(Finite::new(0.79912314).unwrap()),
					},
				],
				line_style: None,
				point_style: None,
				title: Some("Production".to_owned()),
			},
		];
		let text = "After logging true values, you can view metrics comparing your model's performance in production vs. training. The app will automatically alert you if a signficant difference is detected.";
		let left = div()
			.child(div().class("index-step-title").child("Monitor metrics."))
			.child(div().class("index-step-text").child(text));
		let accuracy = div().style(style::GRID_AREA, "accuracy").child(
			ui::Card::new().child(Dehydrate::new(
				"production-accuracy",
				LineChart::new()
					.labels(Some(month_labels.clone()))
					.series(Some(accuracy_data))
					.title("Monthly Accuracy".to_owned())
					.x_axis_grid_line_interval(Some(GridLineInterval { k: 1.0, p: 0.0 }))
					.y_max(Some(Finite::new(1.0).unwrap()))
					.y_min(Some(Finite::new(0.0).unwrap())),
			)),
		);
		let precision = div().style(style::GRID_AREA, "precision").child(
			ui::Card::new().child(Dehydrate::new(
				"production-precision",
				LineChart::new()
					.labels(Some(month_labels))
					.series(Some(precision_data))
					.title("Monthly Precision".to_owned())
					.x_axis_grid_line_interval(Some(GridLineInterval { k: 1.0, p: 0.0 }))
					.y_max(Some(Finite::new(1.0).unwrap()))
					.y_min(Some(Finite::new(0.0).unwrap())),
			)),
		);
		let right = ui::Window::new().child(
			div()
				.class("production-metrics-wrapper")
				.child(accuracy)
				.child(precision),
		);
		div()
			.class("index-step")
			.child(left)
			.child(right)
			.into_node()
	}
}
