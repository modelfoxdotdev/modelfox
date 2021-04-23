use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	metrics_row::MetricsRow,
	tokens::{BASELINE_COLOR, TRAINING_COLOR},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct BinaryClassifierProps {
	pub warning: Option<String>,
	pub positive_class: String,
	pub negative_class: String,
	pub target_column_name: String,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub auc_roc: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
	pub confusion_matrix_section_props: ConfusionMatrixSectionProps,
}

#[component]
pub fn BinaryClassifier(props: BinaryClassifierProps) {
	let aucroc_description = "The area under the receiver operating characteric curve is the probability that a randomly chosen positive example's predicted score is higher than a randomly selected negative example's score. A value of 100% means your model is perfectly able to classify positive and negative rows. A value of 50% means your model is unable to distinguish positive rows from negative rows. A value of 0% means your model is perfectly mis-classifying positive rows as negative and negative rows as positive.";
	html! {
		<ui::S1>
			{props.warning.map(|warning| {
				html! {
					<ui::Alert level={ui::Level::Danger} title?="BAD MODEL">
						{warning}
					</ui::Alert>
				}
			})}
			<ui::H1>{"Training Metrics"}</ui::H1>
			<ui::TabBar>
				<ui::TabLink
					href=""
					selected={true}
				>
					{"Overview"}
				</ui::TabLink>
				<ui::TabLink
					selected={false}
					href="precision_recall"
				>
					{"PR Curve"}
				</ui::TabLink>
				<ui::TabLink
					href="roc"
					selected={false}
				>
					{"ROC Curve"}
				</ui::TabLink>
			</ui::TabBar>
			<ui::S2>
				<ui::P>
					{" The positive class is "}
					<b>{props.positive_class.clone()}</b>
					{" and the negative class is "}
					<b>{props.negative_class.clone()}</b>
				</ui::P>
			</ui::S2>
			<ui::S2>
				<ui::H2>{"Accuracy"}</ui::H2>
				<ui::P>{"Accuracy is the percentage of predictions that were correct."}</ui::P>
				<ui::NumberComparisonCard
					color_a={Some(BASELINE_COLOR.to_owned())}
					color_b={Some(TRAINING_COLOR.to_owned())}
					title="Accuracy"
					value_a={Some(props.baseline_accuracy)}
					value_a_title="Baseline"
					value_b={Some(props.accuracy)}
					value_b_title="Training"
					number_formatter={ui::NumberFormatter::Percent(Default::default())}
				/>
			</ui::S2>
			<ui::S2>
				<ui::H2>{"Area Under the Receiver Operating Characteristic Curve"}</ui::H2>
				<ui::P>{aucroc_description}</ui::P>
				<ui::NumberCard
					title="AUC ROC"
					value={ui::format_percent(props.auc_roc)}
				/>
			</ui::S2>
			<ui::S2>
				<ui::H2>{"Precison, Recall, and F1 Score"}</ui::H2>
				<ui::P>
					{format!("Precision is the percent of rows in the test dataset that the model classified as having \"{}\" equal to ", props.target_column_name)}
					<b>{props.positive_class.clone()}</b>
					{format!(" that actually had \"{}\" equal to ", props.target_column_name)}
					<b>{props.positive_class.clone()}</b>
					{format!(". Recall is the percent of rows in the test dataset that had \"{}\" equal to ", props.target_column_name)}
					<b>{props.positive_class.clone()}</b>
					{" that the model correctly classified as "}
					<b>{props.positive_class.clone()}</b>
					{"."}
				</ui::P>
				<MetricsRow>
					<ui::NumberCard
						title="Precision"
						value={ui::format_percent(props.precision)}
					/>
					<ui::NumberCard
						title="Recall"
						value={ui::format_percent(props.recall)}
					/>
					<ui::NumberCard
						title="F1 Score"
						value={ui::format_percent(props.f1_score)}
					/>
				</MetricsRow>
			</ui::S2>
			<ConfusionMatrixSection {props.confusion_matrix_section_props} />
		</ui::S1>
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
