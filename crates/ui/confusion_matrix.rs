use crate as ui;
use modelfox_number_formatter::PercentFormatter;
use num::ToPrimitive;
use pinwheel::prelude::*;

// |---------------------------------------------------------|
// |           ||     |                Actual                |
// |===========||============================================|
// |           ||     |       Pos        |       Neg         |
// |           ||--------------------------------------------|
// |           ||     |                  |                   |
// |           || Pos |  True Positives  |  False Positives  |
// |           ||     |                  |                   |
// | Predicted ||-----|--------------------------------------|
// |           ||     |                  |                   |
// |           || Neg |  False Negatives |  True Negatives   |
// |           ||     |                  |                   |
// |---------------------------------------------------------|

pub struct ConfusionMatrix {
	pub class_label: String,
	pub false_negatives: Option<usize>,
	pub false_positives: Option<usize>,
	pub true_negatives: Option<usize>,
	pub true_positives: Option<usize>,
}

impl Component for ConfusionMatrix {
	fn into_node(self) -> Node {
		let total = if let (
			Some(false_negatives),
			Some(false_positives),
			Some(true_negatives),
			Some(true_positives),
		) = (
			self.false_negatives,
			self.false_positives,
			self.true_negatives,
			self.true_positives,
		) {
			Some(true_positives + true_negatives + false_positives + false_negatives)
		} else {
			None
		};
		div()
			.class("confusion-matrix-wrapper")
			.child(
				ConfusionMatrixLabel::new("actual-true-label".to_owned())
					.child(div().child("Actual"))
					.child(ui::Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("actual-false-label".to_owned())
					.child(div().child("Actual Not"))
					.child(ui::Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("predicted-true-label".to_owned())
					.left(true)
					.child(div().child("Predicted"))
					.child(ui::Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("predicted-false-label".to_owned())
					.left(true)
					.child(div().child("Predicted Not"))
					.child(ui::Token::new().child(self.class_label)),
			)
			.child(ConfusionMatrixItem {
				area: "true-positive".to_owned(),
				correct: true,
				title: "True Positives".to_owned(),
				total,
				value: self.true_positives,
			})
			.child(ConfusionMatrixItem {
				area: "false-positive".to_owned(),
				correct: false,
				title: "False Positives".to_owned(),
				total,
				value: self.false_positives,
			})
			.child(ConfusionMatrixItem {
				area: "false-negative".to_owned(),
				correct: false,
				title: "False Negatives".to_owned(),
				total,
				value: self.false_negatives,
			})
			.child(ConfusionMatrixItem {
				area: "true-negative".to_owned(),
				correct: true,
				title: "True Negatives".to_owned(),
				total,
				value: self.true_negatives,
			})
			.into_node()
	}
}

pub struct ConfusionMatrixItem {
	area: String,
	correct: bool,
	title: String,
	total: Option<usize>,
	value: Option<usize>,
}

impl Component for ConfusionMatrixItem {
	fn into_node(self) -> Node {
		let class = match self.correct {
			true => "confusion-matrix-item-correct-wrapper",
			false => "confusion-matrix-item-incorrect-wrapper",
		};
		let percent = if let (Some(value), Some(total)) = (self.value, self.total) {
			Some(value.to_f32().unwrap() / total.to_f32().unwrap())
		} else {
			None
		};
		let value = self
			.value
			.map(|value| value.to_string())
			.unwrap_or_else(|| "N/A".to_owned());
		let percent = PercentFormatter::default().format_option(percent);
		div()
			.class("confusion-matrix-item-wrapper")
			.class(class)
			.style(style::GRID_AREA, self.area)
			.child(div().class("confusion-matrix-item-title").child(self.title))
			.child(div().class("confusion-matrix-item-value").child(value))
			.child(div().class("confusion-matrix-item-percent").child(percent))
			.into_node()
	}
}

#[derive(builder, children)]
pub struct ConfusionMatrixLabel {
	pub area: String,
	#[builder]
	pub left: Option<bool>,
	pub children: Vec<Node>,
}

impl ConfusionMatrixLabel {
	pub fn new(area: String) -> ConfusionMatrixLabel {
		ConfusionMatrixLabel {
			area,
			left: None,
			children: Vec::new(),
		}
	}
}

impl Component for ConfusionMatrixLabel {
	fn into_node(self) -> Node {
		let left = self.left.unwrap_or(false);
		let justify_items = if left { "end" } else { "auto" };
		div()
			.class("confusion-matrix-label")
			.style(style::GRID_AREA, self.area)
			.style(style::JUSTIFY_ITEMS, justify_items)
			.child(self.children)
			.into_node()
	}
}
