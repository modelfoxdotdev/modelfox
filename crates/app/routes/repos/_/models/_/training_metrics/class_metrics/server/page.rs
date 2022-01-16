use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::{class_select_field::ClassSelectField, metrics_row::MetricsRow};
use tangram_ui as ui;

pub struct Page {
	pub classes: Vec<String>,
	pub class: String,
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub precision_recall_section: PrecisionRecallSection,
	pub confusion_matrix_section: ConfusionMatrixSection,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_app_training_class_metrics_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new("Training Metrics"))
						.child(
							ui::TabBar::new()
								.child(ui::TabLink::new("./".to_owned(), false).child("Overview"))
								.child(
									ui::TabLink::new("class_metrics".to_owned(), true)
										.child("Class Metrics"),
								),
						)
						.child(
							ui::Form::new()
								.child(ClassSelectField {
									class: self.class.clone(),
									classes: self.classes,
								})
								.child(noscript().child(ui::Button::new().child("Submit"))),
						)
						.child(self.precision_recall_section)
						.child(self.confusion_matrix_section),
				),
			)
			.into_node()
	}
}

pub struct PrecisionRecallSection {
	pub class: String,
	pub f1_score: f32,
	pub precision: f32,
	pub recall: f32,
}

impl Component for PrecisionRecallSection {
	fn into_node(self) -> Node {
		let precision_recall_definition = "Precision is the percentage of examples that were labeled as this class that are actually this class. Recall is the percentage of examples that are of this class that were labeled as this class.";
		ui::S2::new()
			.child(ui::H2::new("Precision and Recall"))
			.child(ui::P::new().child(precision_recall_definition))
			.child(
				MetricsRow::new()
					.child(ui::NumberCard::new(
						"Precision".to_owned(),
						ui::format_percent(self.precision),
					))
					.child(ui::NumberCard::new(
						"Recall".to_owned(),
						ui::format_percent(self.recall),
					)),
			)
			.child(MetricsRow::new().child(ui::NumberCard::new(
				"F1 Score".to_owned(),
				ui::format_percent(self.f1_score),
			)))
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
		let confusion_matrix_definition = "A confusion matrix categorizes predictions into false negatives, false positives, true negatives, and true positives.";
		ui::S2::new()
			.child(ui::H2::new("Confusion Matrix"))
			.child(ui::P::new().child(confusion_matrix_definition))
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
