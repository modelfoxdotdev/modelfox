use pinwheel::prelude::*;
use modelfox_app_ui::colors::{BASELINE_COLOR, TRAINING_COLOR};
use modelfox_ui as ui;
use modelfox_zip::zip;

pub struct MulticlassClassifier {
	pub warning: Option<String>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub class_metrics: Vec<ClassMetrics>,
	pub classes: Vec<String>,
}

pub struct ClassMetrics {
	pub precision: f32,
	pub recall: f32,
}

impl Component for MulticlassClassifier {
	fn into_node(self) -> Node {
		let precision_definition = "Precision is the percentage of examples that were labeled as this class that are actually this class. Recall is the percentage of examples that are of this class that were labeled as this class.";
		ui::S1::new()
			.child(ui::H1::new("Training Metrics"))
			.child(
				ui::TabBar::new()
					.child(ui::TabLink::new("".to_owned(), true).child("Overview"))
					.child(
						ui::TabLink::new("class_metrics".to_owned(), false).child("Class Metrics"),
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
					.child(ui::H2::new("Precision and Recall"))
					.child(ui::P::new().child(precision_definition))
					.child(
						ui::Table::new()
							.width("100%".to_owned())
							.child(
								ui::TableHeader::new().child(
									ui::TableRow::new()
										.child(ui::TableHeaderCell::new().child("Class"))
										.child(ui::TableHeaderCell::new().child("Precision"))
										.child(ui::TableHeaderCell::new().child("Recall")),
								),
							)
							.child(ui::TableBody::new().children(
								zip!(self.class_metrics, self.classes).map(
									|(class_metrics, class_name)| {
										ui::TableRow::new()
											.child(ui::TableCell::new().child(class_name))
											.child(
												ui::TableCell::new().child(ui::format_percent(
													class_metrics.precision,
												)),
											)
											.child(
												ui::TableCell::new().child(ui::format_percent(
													class_metrics.recall,
												)),
											)
									},
								),
							)),
					),
			)
			.into_node()
	}
}
