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
	pub id: String,
	pub roc_curve_data: Vec<RocCurveData>,
	pub model_layout_props: ModelLayoutProps,
	pub class: String,
	pub auc_roc: f32,
}

pub struct RocCurveData {
	pub false_positive_rate: f32,
	pub true_positive_rate: f32,
}

#[component]
pub fn Page(props: PageProps) {
	let aucroc_description = "The area under the receiver operating characteric curve is the probability that a randomly chosen positive example's predicted score is higher than a randomly selected negative example's score. A value of 100% means your model is perfectly able to classify positive and negative rows. A value of 50% means your model is unable to distinguish positive rows from negative rows. A value of 0% means your model is perfectly mis-classifying positive rows as negative and negative rows as positive.";
	let roc_description = "The Receiver Operating Characteristic Curve shows the True Positive Rate v. False Positive Rate at various thresholds.";
	let roc_series = props
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
							selected={false}
						>
							{"PR Curve"}
						</ui::TabLink>
						<ui::TabLink
							href="roc"
							selected={true}
						>
							{"ROC Curve"}
						</ui::TabLink>
					</ui::TabBar>
					<ui::S2>
						<ui::H2>{"Area Under the Receiver Operating Characteristic"}</ui::H2>
						<ui::P>{aucroc_description}</ui::P>
						<ui::NumberCard
							title="AUC ROC"
							value={ui::format_percent(props.auc_roc)}
						/>
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Receiver Operating Characteristic Curve"}</ui::H2>
						<ui::P>{roc_description}</ui::P>
						<ui::Card>
							<LineChart
								id?="roc"
								series?={Some(roc_series)}
								title?="Receiver Operating Characteristic Curve"
								x_axis_title?="False Positive Rate"
								y_axis_title?="True Positive Rate"
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
