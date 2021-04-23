use crate::{Card, Token};
use html::{classes, component, html, Props};
use tangram_number_formatter::NumberFormatter;
use wasm_bindgen::JsCast;
use web_sys::*;

#[derive(Props)]
pub struct NumberComparisonCardProps {
	#[optional]
	pub id: Option<String>,
	pub color_a: Option<String>,
	pub color_b: Option<String>,
	pub title: Option<String>,
	pub value_a: Option<f32>,
	pub value_a_title: Option<String>,
	pub value_b: Option<f32>,
	pub value_b_title: String,
	pub number_formatter: NumberFormatter,
}

#[component]
pub fn NumberComparisonCard(props: NumberComparisonCardProps) {
	let number_formatter = props.number_formatter;
	let number_formatter_string = serde_json::to_string(&number_formatter).unwrap();
	let difference_string = difference_string(props.value_a, props.value_b, &number_formatter);
	let difference_class = difference_class(props.value_a, props.value_b);
	let difference_class = classes!("number-comparison-card-difference", difference_class);
	let value_a = number_formatter.format_option(props.value_a);
	let value_b = number_formatter.format_option(props.value_b);
	html! {
		<Card>
			<div class="number-comparison-card-wrapper"
				id={props.id}
				data-number-formatter={number_formatter_string}
			>
				<div class="number-comparison-card-title">{props.title}</div>
				<div class={difference_class} data-field="difference">{difference_string}</div>
				<div class="number-comparison-card-value a" data-field="value-a">{value_a}</div>
				<div class="number-comparison-card-value b" data-field="value-b">{value_b}</div>
				<div class="number-comparison-card-value-title a">
					<Token color?={props.color_a}>{props.value_a_title}</Token>
				</div>
				<div class="number-comparison-card-value-title b">
					<Token color?={props.color_b}>{props.value_b_title}</Token>
				</div>
			</div>
		</Card>
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

pub fn update_number_comparison_chart(id: &str, value_a: Option<f32>, value_b: Option<f32>) {
	let document = window().unwrap().document().unwrap();
	let container = document
		.get_element_by_id(&id)
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let number_formatter = container.dataset().get("numberFormatter").unwrap();
	let number_formatter: NumberFormatter = serde_json::from_str(&number_formatter).unwrap();
	let difference_element = document
		.query_selector(&format!("#{} [data-field='difference']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let value_a_element = document
		.query_selector(&format!("#{} [data-field='value-a']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let value_b_element = document
		.query_selector(&format!("#{} [data-field='value-b']", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	value_a_element.set_inner_html(&number_formatter.format_option(value_a));
	value_b_element.set_inner_html(&number_formatter.format_option(value_b));
	difference_element.set_inner_html(&difference_string(value_a, value_b, &number_formatter));
	let difference_class = difference_class(value_a, value_b);
	let difference_class = classes!("number-comparison-card-difference", difference_class);
	difference_element.set_class_name(&difference_class);
}
