use num::ToPrimitive;
use pinwheel::prelude::*;
use modelfox_app_ui::metrics_row::MetricsRow;
use modelfox_charts::{
	box_chart::BoxChartPoint,
	box_chart::{BoxChartSeries, BoxChartValue},
	components::BoxChart,
};
use modelfox_ui as ui;

pub struct NumberColumn {
	pub invalid_count: u64,
	pub max: f32,
	pub mean: f32,
	pub min: f32,
	pub name: String,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub std: f32,
	pub unique_count: u64,
}

impl Component for NumberColumn {
	fn into_node(self) -> Node {
		let quantiles_chart_series = vec![BoxChartSeries {
			color: ui::colors::BLUE.to_string(),
			data: vec![BoxChartPoint {
				label: self.name.clone(),
				x: 0.0,
				y: Some(BoxChartValue {
					max: self.max.to_f64().unwrap(),
					min: self.min.to_f64().unwrap(),
					p25: self.p25.to_f64().unwrap(),
					p50: self.p50.to_f64().unwrap(),
					p75: self.p75.to_f64().unwrap(),
				}),
			}],
			title: Some("quartiles".to_owned()),
		}];
		let number_quantiles_title = Some(format!("Distribution of Values for {}", self.name));
		ui::S1::new()
			.child(ui::H1::new(self.name.clone()))
			.child(
				ui::S2::new()
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"Unique Count".to_owned(),
								self.unique_count.to_string(),
							))
							.child(ui::NumberCard::new(
								"Invalid Count".to_owned(),
								self.invalid_count.to_string(),
							)),
					)
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"Mean".to_owned(),
								ui::format_float(self.mean),
							))
							.child(ui::NumberCard::new(
								"Standard Deviation".to_owned(),
								ui::format_float(self.std),
							)),
					)
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"Min".to_owned(),
								ui::format_float(self.min),
							))
							.child(ui::NumberCard::new(
								"Max".to_owned(),
								ui::format_float(self.max),
							)),
					)
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"p25".to_owned(),
								ui::format_float(self.p25),
							))
							.child(ui::NumberCard::new(
								"p50 (median)".to_owned(),
								ui::format_float(self.p50),
							))
							.child(ui::NumberCard::new(
								"p75".to_owned(),
								ui::format_float(self.p75),
							)),
					)
					.child(
						ui::Card::new().child(Dehydrate::new(
							"number_quantiles",
							BoxChart::new()
								.series(quantiles_chart_series)
								.title(number_quantiles_title),
						)),
					),
			)
			.into_node()
	}
}
