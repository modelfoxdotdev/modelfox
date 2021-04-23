use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::metrics_row::MetricsRow;
use tangram_charts::{
	box_chart::BoxChartPoint,
	box_chart::{BoxChartSeries, BoxChartValue},
	components::BoxChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct NumberColumnProps {
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

#[component]
pub fn NumberColumn(props: NumberColumnProps) {
	let quantiles_chart_series = vec![BoxChartSeries {
		color: ui::colors::BLUE.to_string(),
		data: vec![BoxChartPoint {
			label: props.name.clone(),
			x: 0.0,
			y: Some(BoxChartValue {
				max: props.max.to_f64().unwrap(),
				min: props.min.to_f64().unwrap(),
				p25: props.p25.to_f64().unwrap(),
				p50: props.p50.to_f64().unwrap(),
				p75: props.p75.to_f64().unwrap(),
			}),
		}],
		title: Some("quartiles".to_owned()),
	}];
	html! {
		<ui::S1>
			<ui::H1>{props.name.clone()}</ui::H1>
			<ui::S2>
				<MetricsRow>
					<ui::NumberCard
						title="Unique Count"
						value={props.unique_count.to_string()}
					/>
					<ui::NumberCard
						title="Invalid Count"
						value={props.invalid_count.to_string()}
					/>
				</MetricsRow>
				<MetricsRow>
					<ui::NumberCard
						title="Mean"
						value={ui::format_float(props.mean)}
					/>
					<ui::NumberCard
						title="Standard Deviation"
						value={ui::format_float(props.std)}
					/>
				</MetricsRow>
				<MetricsRow>
					<ui::NumberCard
						title="Min"
						value={ui::format_float(props.min)}
					/>
					<ui::NumberCard
						title="Max"
						value={ui::format_float(props.max)}
					/>
				</MetricsRow>
				<MetricsRow>
					<ui::NumberCard
						title="p25"
						value={ui::format_float(props.p25)}
					/>
					<ui::NumberCard
						title="p50 (median)"
						value={ui::format_float(props.p50)}
					/>
					<ui::NumberCard
						title="p75"
						value={ui::format_float(props.p75)}
					/>
				</MetricsRow>
				<ui::Card>
				<BoxChart
					id?="number_quantiles"
					series?={Some(quantiles_chart_series)}
					title?={Some(format!("Distribution of Values for {}", props.name))}
				/>
				</ui::Card>
			</ui::S2>
		</ui::S1>
	}
}
