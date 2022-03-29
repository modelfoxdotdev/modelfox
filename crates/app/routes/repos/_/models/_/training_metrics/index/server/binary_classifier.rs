use num::ToPrimitive;
use pinwheel::prelude::*;
use modelfox_app_ui::{
	colors::{BASELINE_COLOR, TRAINING_COLOR},
	metrics_row::MetricsRow,
};
use modelfox_ui as ui;

pub struct BinaryClassifier {
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
	pub confusion_matrix_section: ConfusionMatrixSection,
}

impl Component for BinaryClassifier {
	fn into_node(self) -> Node {
		let aucroc_description = "The area under the receiver operating characteric curve is the probability that a randomly chosen positive example's predicted score is higher than a randomly selected negative example's score. A value of 100% means your model is perfectly able to classify positive and negative rows. A value of 50% means your model is unable to distinguish positive rows from negative rows. A value of 0% means your model is perfectly mis-classifying positive rows as negative and negative rows as positive.";
		let definition = ui::P::new().child(
				format!("Precision is the percent of rows in the test dataset that the model classified as having \"{}\" equal to ", self.target_column_name)
			).child(
				b().child(self.positive_class.clone())
			).child(
				format!(" that actually had \"{}\" equal to ", self.target_column_name)
			).child(
				b().child(self.positive_class.clone())
			)
			.child(
				format!(". Recall is the percent of rows in the test dataset that had \"{}\" equal to ", self.target_column_name)
			)
			.child(b().child(self.positive_class.clone()))
			.child(" that the model correctly classified as ")
			.child(b().child(self.positive_class.clone()))
			.child(".")
			.into_node();
		ui::S1::new()
			.child(self.warning.map(|warning| {
				ui::Alert::new(ui::Level::Danger)
					.title("BAD MODEL".to_owned())
					.child(warning)
			}))
			.child(ui::H1::new("Training Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("".to_owned(), true).child("Overview"))
					.child(ui::TabLink::new("precision_recall".to_owned(), false).child("PR Curve"))
					.child(ui::TabLink::new("roc".to_owned(), false).child("ROC Curve")),
			)
			.child(
				ui::S2::new().child(
					ui::P::new()
						.child(" The positive class is ")
						.child(b().child(self.positive_class.clone()))
						.child(" and the negative class is ")
						.child(b().child(self.negative_class.clone())),
				),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new("Accuracy"))
					.child(
						ui::P::new()
							.child("Accuracy is the percentage of predictions that were correct."),
					)
					.child(
						ui::NumberComparisonCard::new(
							Some(self.baseline_accuracy),
							Some(self.accuracy),
						)
						.color_a(BASELINE_COLOR.to_owned())
						.color_b(TRAINING_COLOR.to_owned())
						.title("Accuracy".to_owned())
						.value_a_title("Baseline".to_owned())
						.value_b_title("Training".to_owned())
						.number_formatter(ui::NumberFormatter::Percent(Default::default())),
					),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new(
						"Area Under the Receiver Operating Characteristic Curve",
					))
					.child(ui::P::new().child(aucroc_description))
					.child(ui::NumberCard::new(
						"AUC ROC".to_owned(),
						ui::format_percent(self.auc_roc),
					)),
			)
			.child(
				ui::S2::new()
					.child(ui::H2::new("Precison, Recall, and F1 Score"))
					.child(definition)
					.child(
						MetricsRow::new()
							.child(ui::NumberCard::new(
								"Precision".to_owned(),
								ui::format_percent(self.precision),
							))
							.child(ui::NumberCard::new(
								"Recall".to_owned(),
								ui::format_percent(self.recall),
							))
							.child(ui::NumberCard::new(
								"F1 Score".to_owned(),
								ui::format_percent(self.f1_score),
							)),
					),
			)
			.child(self.confusion_matrix_section)
			.into_node()
	}
}

pub struct ConfusionMatrixSection {
	pub class: String,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub true_positives: u64,
}

impl Component for ConfusionMatrixSection {
	fn into_node(self) -> Node {
		let definition = "A confusion matrix categorizes predictions into false negatives, false positives, true negatives, and true positives.";
		ui::S2::new()
			.child(ui::H2::new("Confusion Matrix"))
			.child(ui::P::new().child(definition))
			.child(ui::ConfusionMatrix {
				class_label: self.class,
				false_negatives: self.false_negatives.to_usize(),
				false_positives: self.false_positives.to_usize(),
				true_negatives: self.true_negatives.to_usize(),
				true_positives: self.true_positives.to_usize(),
			})
			.into_node()
	}
}
