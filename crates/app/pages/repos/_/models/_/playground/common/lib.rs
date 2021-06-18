use pinwheel::prelude::*;
use tangram_charts::components::{BarChart, BoxChart};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ColumnChart {
	#[serde(rename = "bar")]
	Bar(BarChart),
	#[serde(rename = "box")]
	Box(BoxChart),
}

impl Component for ColumnChart {
	fn into_node(self) -> Node {
		let chart = match self {
			ColumnChart::Bar(bar_chart) => bar_chart.into_node(),
			ColumnChart::Box(box_chart) => box_chart.into_node(),
		};
		div()
			.class("predict-column-chart-wrapper")
			.child(chart)
			.into_node()
	}
}
