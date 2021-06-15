use pinwheel::prelude::*;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	box_chart::{BoxChartPoint, BoxChartSeries, BoxChartValue},
	components::{BarChart, BoxChart},
};
use tangram_ui as ui;

pub struct ProductionStats;

impl Component for ProductionStats {
	fn into_node(self) -> Node {
		let bar_chart_series = vec![
			BarChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: vec![
					BarChartPoint {
						label: "asymptomatic".to_owned(),
						x: 0.0,
						y: Some(0.4752),
					},
					BarChartPoint {
						label: "atypical angina".to_owned(),
						x: 1.0,
						y: Some(0.165),
					},
					BarChartPoint {
						label: "non-angina pain".to_owned(),
						x: 2.0,
						y: Some(0.2838),
					},
					BarChartPoint {
						label: "typical angina".to_owned(),
						x: 3.0,
						y: Some(0.07591),
					},
				],
				title: Some("Training".to_owned()),
			},
			BarChartSeries {
				color: ui::colors::GREEN.to_owned(),
				data: vec![
					BarChartPoint {
						label: "asymptomatic".to_owned(),
						x: 0.0,
						y: Some(0.0),
					},
					BarChartPoint {
						label: "atypical angina".to_owned(),
						x: 1.0,
						y: Some(0.1622),
					},
					BarChartPoint {
						label: "non-angina pain".to_owned(),
						x: 2.0,
						y: Some(0.2903),
					},
					BarChartPoint {
						label: "typical angina".to_owned(),
						x: 3.0,
						y: Some(0.07508),
					},
				],
				title: Some("Production".to_owned()),
			},
		];
		let box_chart_series = vec![
			BoxChartSeries {
				color: ui::colors::BLUE.to_owned(),
				data: vec![BoxChartPoint {
					label: "exercise_max_heart_rate".to_owned(),
					x: 0.0,
					y: Some(BoxChartValue {
						max: 202.0,
						min: 71.0,
						p25: 133.5,
						p50: 153.0,
						p75: 167.5,
					}),
				}],
				title: Some("Training".to_owned()),
			},
			BoxChartSeries {
				color: ui::colors::GREEN.to_owned(),
				data: vec![BoxChartPoint {
					label: "exercise_max_heart_rate".to_owned(),
					x: 0.0,
					y: Some(BoxChartValue {
						max: 196.2,
						min: 59.0,
						p25: 120.5,
						p50: 143.1,
						p75: 177.5,
					}),
				}],
				title: Some("Production".to_owned()),
			},
		];
		let title = div().class("index-step-title").child("Monitor data drift.");
		let text = "After logging predictions, you can view stats comparing the production data with the training data. The app will automatically alert you if a significant difference is detected.";
		let left = div()
			.child(title)
			.child(div().class("index-step-text").child(text));
		let number_alert = div()
			.style(style::GRID_AREA, "number-alert")
			.child(ui::Alert::new(ui::Level::Success).child("All Good"));
		let number_chart = div().style(style::GRID_AREA, "number").child(
			ui::Card::new().child(Dehydrate::new(
				"production-stats-number",
				BoxChart::new()
					.series(Some(box_chart_series))
					.title("exercise_max_heart_rate".to_owned())
					.should_draw_x_axis_labels(Some(false)),
			)),
		);
		let enum_alert = div()
			.style(style::GRID_AREA, "enum-alert")
			.child(ui::Alert::new(ui::Level::Warning).child("High Invalid Count"));
		let enum_chart = div().style(style::GRID_AREA, "enum").child(
			ui::Card::new().child(Dehydrate::new(
				"production-stats-enum",
				BarChart::new()
					.series(Some(bar_chart_series))
					.title("chest_pain".to_owned())
					.x_axis_title("chest_pain".to_owned())
					.y_axis_title("Percent".to_owned())
					.y_max(Some(1.0))
					.y_min(Some(0.0)),
			)),
		);
		let right = ui::Window::new().child(
			div()
				.class("production-stats-wrapper")
				.child(number_alert)
				.child(number_chart)
				.child(enum_alert)
				.child(enum_chart),
		);
		div()
			.class("index-step")
			.child(left)
			.child(right)
			.into_node()
	}
}
