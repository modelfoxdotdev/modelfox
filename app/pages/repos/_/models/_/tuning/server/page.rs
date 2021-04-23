use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::tokens::{BASELINE_COLOR, SELECTED_THRESHOLD_COLOR};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
pub use tangram_app_tuning_common::{ClientProps, Metrics};
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub model_layout_props: ModelLayoutProps,
	pub tuning: Option<TuningProps>,
}

#[derive(Props)]
pub struct TuningProps {
	pub default_threshold: f32,
	pub metrics: Vec<Metrics>,
	pub default_threshold_metrics: Metrics,
	pub class: String,
}

#[component]
pub fn Page(props: PageProps) {
	let inner = match props.tuning {
		Some(tuning_props) => html! {
			<Tuning {tuning_props} />
		},
		None => html! {
			<ui::S1>
				<ui::P>{"Tuning is not supported for this model."}</ui::P>
			</ui::S1>
		},
	};
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				{inner}
			</ModelLayout>
		</Document>
	}
}

#[component]
fn Tuning(props: TuningProps) {
	let thresholds = props
		.metrics
		.iter()
		.map(|metric| metric.threshold)
		.collect::<Vec<_>>();
	let baseline_index = thresholds
		.iter()
		.position(|value| (value - props.default_threshold).abs() < std::f32::EPSILON)
		.unwrap();
	let selected_threshold_index = baseline_index;
	let selected_threshold = thresholds[selected_threshold_index];
	let baseline_metrics = &props.default_threshold_metrics;
	let selected_threshold_metrics = &props.metrics[selected_threshold_index];
	let client_props = ClientProps {
		baseline_metrics: baseline_metrics.clone(),
		threshold_metrics: props.metrics.clone(),
	};
	let client_props = serde_json::to_string(&client_props).unwrap();
	html! {
		<div id="tuning_page" data-props={client_props}>
			<ui::S1>
				<ui::H1>{"Tuning"}</ui::H1>
				<ui::P>
					{"Drag the silder to choose a threshold."}
				</ui::P>
				<ui::Slider
					id="tuning_slider"
					max={(thresholds.len() - 1).to_f32().unwrap()}
					min={0.0}
					value={selected_threshold_index}
				/>
				{if selected_threshold == 0.0 {
					Some(html! {
						<ui::Alert level={ui::Level::Info}>
							{"A threshold of 0 makes your model predict the same class for every input."}
						</ui::Alert>
					})
				} else if selected_threshold.partial_cmp(&1.0).unwrap() == std::cmp::Ordering::Equal {
					Some(html! {
						<ui::Alert level={ui::Level::Info}>
							{"A threshold of 1 makes your model predict the same class for every input."}
						</ui::Alert>
					})
				} else {
					None
				}}
				<ui::NumberCard
					id?="tuning-threshold"
					title="Selected Threshold"
					value={selected_threshold.to_string()}
				/>
				<div class="tuning-metrics-grid">
					<ui::NumberComparisonCard
						id?="tuning-accuracy"
						color_a={Some(BASELINE_COLOR.to_owned())}
						color_b={Some(SELECTED_THRESHOLD_COLOR.to_owned())}
						title="Accuracy"
						value_a={Some(baseline_metrics.accuracy)}
						value_a_title="Baseline"
						value_b={Some(selected_threshold_metrics.accuracy)}
						value_b_title="Selected Threshold"
						number_formatter={ui::NumberFormatter::Percent(Default::default())}
					/>
					<ui::NumberComparisonCard
						id?="tuning-f1-score"
						color_a={Some(BASELINE_COLOR.to_owned())}
						color_b={Some(SELECTED_THRESHOLD_COLOR.to_owned())}
						title="F1 Score"
						value_a={Some(baseline_metrics.f1_score.unwrap())}
						value_a_title="Baseline"
						value_b={selected_threshold_metrics.f1_score}
						value_b_title="Selected Threshold"
						number_formatter={ui::NumberFormatter::Percent(Default::default())}
					/>
					<ui::NumberComparisonCard
						id?="tuning-precision"
						color_a={Some(BASELINE_COLOR.to_owned())}
						color_b={Some(SELECTED_THRESHOLD_COLOR.to_owned())}
						title="Precision"
						value_a={baseline_metrics.precision}
						value_a_title="Baseline"
						value_b={selected_threshold_metrics.precision}
						value_b_title="Selected Threshold"
						number_formatter={ui::NumberFormatter::Percent(Default::default())}
					/>
					<ui::NumberComparisonCard
						id?="tuning-recall"
						color_a={Some(BASELINE_COLOR.to_owned())}
						color_b={Some(SELECTED_THRESHOLD_COLOR.to_owned())}
						title="Recall"
						value_a={baseline_metrics.recall}
						value_a_title="Baseline"
						value_b={selected_threshold_metrics.recall}
						value_b_title="Selected Threshold"
						number_formatter={ui::NumberFormatter::Percent(Default::default())}
					/>
				</div>
				<ui::ConfusionMatrixComparison
					id?="tuning-confusion-matrix-comparison"
					class_label={props.class.to_owned()}
					color_a={BASELINE_COLOR.to_owned()}
					color_b={SELECTED_THRESHOLD_COLOR.to_owned()}
					value_a={Some(ui::ConfusionMatrixComparisonValue {
						false_negative: baseline_metrics.false_negatives_fraction,
						false_positive: baseline_metrics.false_positives_fraction,
						true_negative: baseline_metrics.true_negatives_fraction,
						true_positive: baseline_metrics.true_positives_fraction,
					})}
					value_a_title="Baseline"
					value_b={Some(ui::ConfusionMatrixComparisonValue {
						false_negative: selected_threshold_metrics.false_negatives_fraction,
						false_positive: selected_threshold_metrics.false_positives_fraction,
						true_negative: selected_threshold_metrics.true_negatives_fraction,
						true_positive: selected_threshold_metrics.true_positives_fraction,
					})}
					value_b_title="Selected Threshold"
				/>
			</ui::S1>
		</div>
	}
}
