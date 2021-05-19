use crate::Token;
use pinwheel::prelude::*;
use tangram_number_formatter::format_option_percent;

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

#[derive(ComponentBuilder)]
pub struct ConfusionMatrixComparison {
	pub class_label: String,
	pub color_a: String,
	pub color_b: String,
	pub value_a: BoxSignal<Option<ConfusionMatrixComparisonValue>>,
	pub value_a_title: String,
	pub value_b: BoxSignal<Option<ConfusionMatrixComparisonValue>>,
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
				ConfusionMatrixLabel::new("actual-true-label", None)
					.child(div().child("Actual"))
					.child(Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("actual-false-label", None)
					.child(div().child("Actual Not"))
					.child(Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("predicted-true-label".to_owned(), Some(true))
					.child(div().child("Predicted"))
					.child(Token::new().child(self.class_label.clone())),
			)
			.child(
				ConfusionMatrixLabel::new("predicted-false-label".to_owned(), Some(true))
					.child(div().child("Predicted Not"))
					.child(Token::new().child(self.class_label)),
			)
			.child(ConfusionMatrixComparisonItem {
				area: "true-positive".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: true,
				label: "True Positives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self
					.value_a
					.signal_cloned()
					.map(|value_a| value_a.as_ref().map(|value| value.true_positive))
					.boxed(),
				value_b_title: self.value_b_title.clone(),
				value_b: self
					.value_b
					.signal_cloned()
					.map(|value_b| value_b.as_ref().map(|value| value.true_positive))
					.boxed(),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "false-positive".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: false,
				label: "False Positives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self
					.value_a
					.signal_cloned()
					.map(|value_a| value_a.as_ref().map(|value| value.false_positive))
					.boxed(),
				value_b_title: self.value_b_title.clone(),
				value_b: self
					.value_b
					.signal_cloned()
					.map(|value_b| value_b.as_ref().map(|value| value.false_positive))
					.boxed(),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "false-negative".to_owned(),
				color_a: self.color_a.clone(),
				color_b: self.color_b.clone(),
				correct: false,
				label: "False Negatives".to_owned(),
				value_a_title: self.value_a_title.clone(),
				value_a: self
					.value_a
					.signal_cloned()
					.map(|value_a| value_a.as_ref().map(|value| value.false_negative))
					.boxed(),
				value_b_title: self.value_b_title.clone(),
				value_b: self
					.value_b
					.signal_cloned()
					.map(|value_b| value_b.as_ref().map(|value| value.false_negative))
					.boxed(),
			})
			.child(ConfusionMatrixComparisonItem {
				area: "true-negative".to_owned(),
				color_a: self.color_a,
				color_b: self.color_b,
				correct: true,
				label: "True Negatives".to_owned(),
				value_a_title: self.value_a_title,
				value_a: self
					.value_a
					.signal_cloned()
					.map(|value_a| value_a.as_ref().map(|value| value.true_negative))
					.boxed(),
				value_b_title: self.value_b_title,
				value_b: self
					.value_b
					.signal_cloned()
					.map(|value_b| value_b.as_ref().map(|value| value.true_negative))
					.boxed(),
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
	pub value_a: BoxSignal<Option<f32>>,
	pub value_b_title: String,
	pub value_b: BoxSignal<Option<f32>>,
}

impl Component for ConfusionMatrixComparisonItem {
	fn into_node(self) -> Node {
		let class = if self.correct {
			"confusion-matrix-comparison-item-correct-wrapper"
		} else {
			"confusion-matrix-comparison-item-incorrect-wrapper"
		};
		let class = classes!("confusion-matrix-comparison-item-wrapper", class);
		let value_a = self.value_a.signal_cloned().map(format_option_percent);
		let value_b = self.value_b.signal_cloned().map(format_option_percent);
		div()
			.attribute("class", class)
			.style(style::GRID_AREA, self.area.clone())
			.attribute("data-area", self.area)
			.child(
				div()
					.class("confusion-matrix-comparison-item-title")
					.child(self.label),
			)
			.child(
				div()
					.attribute(
						"class",
						"confusion-matrix-comparison-number-comparison-wrapper",
					)
					.child(
						div()
							.class("confusion-matrix-comparison-item-value")
							.attribute("data-field", "value-a")
							.child_signal(value_a),
					)
					.child(
						div()
							.class("confusion-matrix-comparison-item-value")
							.attribute("data-field", "value-b")
							.child_signal(value_b),
					)
					.child(
						div().child(
							Token::new()
								.color(Some(self.color_a))
								.child(self.value_a_title),
						),
					)
					.child(
						div().child(
							Token::new()
								.color(Some(self.color_b))
								.child(self.value_b_title),
						),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ConfusionMatrixLabel {
	pub area: String,
	pub left: Option<bool>,
	#[children]
	children: Vec<Node>,
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
