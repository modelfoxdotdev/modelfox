use crate as ui;
use pinwheel::prelude::*;
use tangram_number_formatter::NumberFormatter;

#[derive(ComponentBuilder)]
pub struct NumberComparisonCard {
	pub value_a: Option<f32>,
	pub value_b: Option<f32>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub color_a: Option<String>,
	#[optional]
	pub color_b: Option<String>,
	#[optional]
	pub number_formatter: NumberFormatter,
	#[optional]
	pub title: Option<String>,
	#[optional]
	pub value_a_title: Option<String>,
	#[optional]
	pub value_b_title: Option<String>,
}

impl Component for NumberComparisonCard {
	fn into_node(self) -> Node {
		let number_formatter = self.number_formatter;
		let number_formatter_string = serde_json::to_string(&number_formatter).unwrap();
		let difference_string = difference_string(self.value_a, self.value_b, &number_formatter);
		let difference_class = classes!(
			"number-comparison-card-difference",
			difference_class(self.value_a, self.value_b)
		);
		let content = div()
			.class("number-comparison-card-wrapper")
			.attribute("id", self.id)
			.attribute("data-number-formatter", number_formatter_string)
			.child(
				div()
					.class("number-comparison-card-title")
					.child(self.title),
			)
			.child(div().class(difference_class).child(difference_string))
			.child(
				div()
					.class("number-comparison-card-value a")
					.child(number_formatter.format_option(self.value_a)),
			)
			.child(
				div()
					.class("number-comparison-card-value b")
					.child(number_formatter.format_option(self.value_b)),
			)
			.child(
				div().class("number-comparison-card-value-title a").child(
					ui::Token::new()
						.color(self.color_a)
						.child(self.value_a_title),
				),
			)
			.child(
				div().class("number-comparison-card-value-title b").child(
					ui::Token::new()
						.color(self.color_b)
						.child(self.value_b_title),
				),
			);
		ui::Card::new().child(content).into_node()
	}
}

fn difference_class(value_a: Option<f32>, value_b: Option<f32>) -> String {
	match (value_a, value_b) {
		(Some(value_a), Some(value_b)) => match value_a.partial_cmp(&value_b) {
			Some(ordering) => match ordering {
				std::cmp::Ordering::Less => "number-comparison-card-positive".to_owned(),
				std::cmp::Ordering::Equal => "number-comparison-card-equal".to_owned(),
				std::cmp::Ordering::Greater => "number-comparison-card-negative".to_owned(),
			},
			None => "number-comparison-card-na".to_owned(),
		},
		(_, _) => "number-comparison-card-na".to_owned(),
	}
}

fn difference_string(
	value_a: Option<f32>,
	value_b: Option<f32>,
	number_formatter: &NumberFormatter,
) -> String {
	if let (Some(value_a), Some(value_b)) = (value_a, value_b) {
		if let Some(ordering) = value_a.partial_cmp(&value_b) {
			return match &ordering {
				std::cmp::Ordering::Less => {
					format!("+{}", number_formatter.format(value_b - value_a))
				}
				std::cmp::Ordering::Equal => "equal".to_owned(),
				std::cmp::Ordering::Greater => number_formatter.format(value_b - value_a),
			};
		}
	}
	"N/A".to_owned()
}
