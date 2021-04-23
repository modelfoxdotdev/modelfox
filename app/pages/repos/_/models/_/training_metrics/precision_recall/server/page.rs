use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub class: String,
	pub precision_recall_curve_series: Vec<PrecisionRecallPoint>,
	pub id: String,
	pub model_layout_props: ModelLayoutProps,
}

pub struct PrecisionRecallPoint {
	pub precision: f32,
	pub recall: f32,
	pub threshold: f32,
}

#[component]
pub fn Page(props: PageProps) {
	let pr_series = props
		.precision_recall_curve_series
		.iter()
		.map(|threshold| LineChartPoint {
			x: Finite::new(threshold.recall.to_f64().unwrap()).unwrap(),
			y: Finite::new(threshold.precision.to_f64().unwrap()).ok(),
		})
		.collect::<Vec<_>>();
	let precision_series = props
		.precision_recall_curve_series
		.iter()
		.map(|threshold| LineChartPoint {
			x: Finite::new(threshold.threshold.to_f64().unwrap()).unwrap(),
			y: Finite::new(threshold.precision.to_f64().unwrap()).ok(),
		})
		.collect::<Vec<_>>();
	let recall_series = props
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
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Training Metrics"}</ui::H1>
					<ui::TabBar>
						<ui::TabLink
							href="./"
							selected={false}
						>
							{"Overview"}
						</ui::TabLink>
						<ui::TabLink
							href="precision_recall"
							selected={true}
						>
							{"PR Curve"}
						</ui::TabLink>
						<ui::TabLink
							selected={false}
							href="roc"
						>
							{"ROC Curve"}
						</ui::TabLink>
					</ui::TabBar>
					<ui::S2>
						<ui::H2>{"Parametric Precision Recall Curve"}</ui::H2>
						<ui::P>{"The parametric precision recall curve shows the value of precision on the y axis for each value of recall on the x axis where each point is at a distinct threshold."}</ui::P>
						<ui::Card>
							<LineChart
								id?="parametric_pr"
								hide_legend?={Some(true)}
								series?={Some(parametric_series)}
								title?="Parametric Precision Recall Curve"
								x_axis_title?="Recall"
								y_axis_title?="Precision"
								x_max?={Some(Finite::new(1.0).unwrap())}
								x_min?={Some(Finite::new(0.0).unwrap())}
								y_max?={Some(Finite::new(1.0).unwrap())}
								y_min?={Some(Finite::new(0.0).unwrap())}
							/>
						</ui::Card>
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Non-Parametric Precision Recall Curve"}</ui::H2>
						<ui::P>{"The non-parametric precision recall curve shows the value of precision and recall the model would get on the y axis for each threshold on the x axis."}</ui::P>
						<ui::Card>
							<LineChart
								id?="non_parametric_pr"
								hide_legend?={Some(true)}
								series?={Some(non_parametric_series)}
								title?="Non-Parametric Precision Recall Curve"
								x_axis_title?="Threshold"
								y_axis_title?="Precision/Recall"
								x_max?={Some(Finite::new(1.0).unwrap())}
								x_min?={Some(Finite::new(0.0).unwrap())}
								y_max?={Some(Finite::new(1.0).unwrap())}
								y_min?={Some(Finite::new(0.0).unwrap())}
							/>
						</ui::Card>
					</ui::S2>
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}
