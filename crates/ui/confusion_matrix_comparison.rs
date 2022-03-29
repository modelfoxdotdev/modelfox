use crate as ui;
use pinwheel::prelude::*;
use modelfox_number_formatter::format_option_percent;

// |-----------------------------------------------------------|
// |           ||       |                Actual                |
// |===========||==============================================|
// |           ||       |       True       |      False        |
// |           ||----------------------------------------------|
// |           ||       |                  |                   |
// |           || True  |  True Positives  |  False Positives  |
// |           ||       |                  |                   |
// | Predicted ||-------|--------------------------------------|
// |           ||       |                  |                   |
// |           || False |  False Negatives |  True Negatives   |
// |           ||       |                  |                   |
// |-----------------------------------------------------------|

#[derive(builder)]
pub struct ConfusionMatrixComparison {
	pub class_label: String,
	pub color_a: String,
	pub color_b: String,
	pub value_a: Option<ConfusionMatrixComparisonValue>,
	pub value_a_title: String,
	pub value_b: Option<ConfusionMatrixComparisonValue>,
	pub value_b_title: String,
}

#[derive(Clone)]
pub struct ConfusionMatrixComparisonValue {
	pub false_negative: f32,
	pub false_positive: f32,
	pub true_negative: f32,
	pub true_positive: f32,
}

impl Component for ConfusionMatrixComparison {
	fn into_node(self) -> Node {
		div()
			.class("confusion-matrix-comparison-wrapper")
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
			.child(ConfusionMatrixComparisonItem {
				area: "true-positive".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: true,
				label: "True Positives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self.value_a.as_ref().map(|value| value.true_positive),
				value_b_title: self.value_b_title.clone(),
				value_b: self.value_b.as_ref().map(|value| value.true_positive),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "false-positive".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: false,
				label: "False Positives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self.value_a.as_ref().map(|value| value.false_positive),
				value_b_title: self.value_b_title.clone(),
				value_b: self.value_b.as_ref().map(|value| value.false_positive),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "false-negative".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: false,
				label: "False Negatives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self.value_a.as_ref().map(|value| value.false_negative),
				value_b_title: self.value_b_title.clone(),
				value_b: self.value_b.as_ref().map(|value| value.false_negative),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "true-negative".to_owned(),
				color_a: self.color_a,
				color_b: self.color_b,
				correct: true,
				label: "True Negatives".to_owned(),
				value_a_title: self.value_a_title,
				value_a: self.value_a.as_ref().map(|value| value.true_negative),
				value_b_title: self.value_b_title,
				value_b: self.value_b.as_ref().map(|value| value.true_negative),
			})
			.into_node()
	}
}

pub struct ConfusionMatrixComparisonItem {
	pub area: String,
	pub color_a: String,
	pub color_b: String,
	pub correct: bool,
	pub label: String,
	pub value_a_title: String,
	pub value_a: Option<f32>,
	pub value_b_title: String,
	pub value_b: Option<f32>,
}

impl Component for ConfusionMatrixComparisonItem {
	fn into_node(self) -> Node {
		let class = if self.correct {
			"confusion-matrix-comparison-item-correct-wrapper"
		} else {
			"confusion-matrix-comparison-item-incorrect-wrapper"
		};
		let value_a = format_option_percent(self.value_a);
		let value_b = format_option_percent(self.value_b);
		div()
			.class("confusion-matrix-comparison-item-wrapper")
			.class(class)
			.style(style::GRID_AREA, self.area.clone())
			.attribute("data-area", self.area)
			.child(
				div()
					.class("confusion-matrix-comparison-item-title")
					.child(self.label),
			)
			.child(
				div()
					.class("confusion-matrix-comparison-number-comparison-wrapper")
					.child(
						div()
							.class("confusion-matrix-comparison-item-value")
							.attribute("data-field", "value-a")
							.child(value_a),
					)
					.child(
						div()
							.class("confusion-matrix-comparison-item-value")
							.attribute("data-field", "value-b")
							.child(value_b),
					)
					.child(
						div().child(
							ui::Token::new()
								.color(self.color_a)
								.child(self.value_a_title),
						),
					)
					.child(
						div().child(
							ui::Token::new()
								.color(self.color_b)
								.child(self.value_b_title),
						),
					),
			)
			.into_node()
	}
}

#[derive(builder, children)]
pub struct ConfusionMatrixLabel {
	area: String,
	#[builder]
	left: Option<bool>,
	children: Vec<Node>,
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
		let justify_items = if left { "end" } else { "center" };
		div()
			.class("confusion-matrix-comparison-label")
			.style(style::GRID_AREA, self.area)
			.style(style::JUSTIFY_ITEMS, justify_items)
			.child(self.children)
			.into_node()
	}
}
