use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{class_select_field::ClassSelectField, metrics_row::MetricsRow};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub classes: Vec<String>,
	pub class: String,
	pub id: String,
	pub model_layout_props: ModelLayoutProps,
	pub precision_recall_section_props: PrecisionRecallSectionProps,
	pub confusion_matrix_section_props: ConfusionMatrixSectionProps,
}

#[component]
pub fn Page(props: PageProps) {
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
							href="class_metrics"
							selected={true}
						>
						{"Class Metrics"}
						</ui::TabLink>
					</ui::TabBar>
					<ui::Form>
						<ClassSelectField class={props.class.clone()} classes={props.classes} />
						<noscript>
							<ui::Button>
								{"Submit"}
							</ui::Button>
						</noscript>
					</ui::Form>
					<PrecisionRecallSection {props.precision_recall_section_props} />
					<ConfusionMatrixSection {props.confusion_matrix_section_props} />
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct PrecisionRecallSectionProps {
	pub class: String,
	pub f1_score: f32,
	pub precision: f32,
	pub recall: f32,
}

#[component]
fn PrecisionRecallSection(props: PrecisionRecallSectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Precision and Recall"}</ui::H2>
			<ui::P>{"Precision is the percentage of examples that were labeled as this class that are actually this class. Recall is the percentage of examples that are of this class that were labeled as this class."}</ui::P>
			<MetricsRow>
				<ui::NumberCard
					title="Precision"
					value={ui::format_percent(props.precision)}
				/>
				<ui::NumberCard
					title="Recall"
					value={ui::format_percent(props.recall)}
				/>
			</MetricsRow>
			<MetricsRow>
				<ui::NumberCard
					title="F1 Score"
					value={ui::format_percent(props.f1_score)}
				/>
			</MetricsRow>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct ConfusionMatrixSectionProps {
	pub class: String,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub true_positives: u64,
}

#[component]
fn ConfusionMatrixSection(props: ConfusionMatrixSectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Confusion Matrix"}</ui::H2>
			<ui::P>{"A confusion matrix categorizes predictions into false negatives, false positives, true negatives, and true positives."}</ui::P>
			<ui::ConfusionMatrix
				class_label={props.class}
				false_negatives={props.false_negatives.to_usize()}
				false_positives={props.false_positives.to_usize()}
				true_negatives={props.true_negatives.to_usize()}
				true_positives={props.true_positives.to_usize()}
			/>
		</ui::S2>
	}
}
