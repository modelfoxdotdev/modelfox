use crate::Token;
use html::{classes, component, html, style, Props};
use tangram_number_formatter::{format_option_percent, format_percent};
use wasm_bindgen::JsCast;
use web_sys::*;

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

#[derive(Props)]
pub struct ConfusionMatrixComparisonProps {
	#[optional]
	pub id: Option<String>,
	pub class_label: String,
	pub color_a: String,
	pub color_b: String,
	pub value_a: Option<ConfusionMatrixComparisonValue>,
	pub value_a_title: String,
	pub value_b: Option<ConfusionMatrixComparisonValue>,
	pub value_b_title: String,
}

pub struct ConfusionMatrixComparisonValue {
	pub false_negative: f32,
	pub false_positive: f32,
	pub true_negative: f32,
	pub true_positive: f32,
}

#[component]
pub fn ConfusionMatrixComparison(props: ConfusionMatrixComparisonProps) {
	html! {
		<div class="confusion-matrix-comparison-wrapper" id={props.id}>
			<ConfusionMatrixLabel area="actual-true-label" left={None}>
				<div>{"Actual"}</div>
				<Token>{props.class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="actual-false-label" left={None}>
				<div>{"Actual Not"}</div>
				<Token>{props.class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-true-label" left={Some(true)}>
				<div>{"Predicted"}</div>
				<Token>{props.class_label.clone()}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixLabel area="predicted-false-label" left={Some(true)}>
				<div>{"Predicted Not"}</div>
				<Token>{props.class_label}</Token>
			</ConfusionMatrixLabel>
			<ConfusionMatrixComparisonItem
				area="true-positive"
				color_a={props.color_a.clone()}
				color_b={props.color_b.clone()}
				label="True Positives"
				correct={true}
				value_a={props.value_a.as_ref().map(|value| value.true_positive)}
				value_a_title={props.value_a_title.clone()}
				value_b={props.value_b.as_ref().map(|value| value.true_positive)}
				value_b_title={props.value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area="false-positive"
				color_a={props.color_a.clone()}
				correct={false}
				color_b={props.color_b.clone()}
				label="False Positives"
				value_a={props.value_a.as_ref().map(|value| value.false_positive)}
				value_a_title={props.value_a_title.clone()}
				value_b={props.value_b.as_ref().map(|value| value.false_positive)}
				value_b_title={props.value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area="false-negative"
				color_a={props.color_a.clone()}
				color_b={props.color_b.clone()}
				correct={false}
				label="False Negatives"
				value_a={props.value_a.as_ref().map(|value| value.false_negative)}
				value_a_title={props.value_a_title.clone()}
				value_b={props.value_b.as_ref().map(|value| value.false_negative)}
				value_b_title={props.value_b_title.clone()}
			/>
			<ConfusionMatrixComparisonItem
				area="true-negative"
				color_a={props.color_a}
				color_b={props.color_b}
				label="True Negatives"
				correct={true}
				value_a={props.value_a.as_ref().map(|value| value.true_negative)}
				value_a_title={props.value_a_title}
				value_b={props.value_b.as_ref().map(|value| value.true_negative)}
				value_b_title={props.value_b_title}
			/>
		</div>
	}
}

#[derive(Props)]
pub struct ConfusionMatrixComparisonItemProps {
	pub area: String,
	pub color_a: String,
	pub color_b: String,
	pub label: String,
	pub correct: bool,
	pub value_a: Option<f32>,
	pub value_a_title: String,
	pub value_b: Option<f32>,
	pub value_b_title: String,
}

#[component]
fn ConfusionMatrixComparisonItem(props: ConfusionMatrixComparisonItemProps) {
	let wrapper_style = style! {
		"grid-area" => props.area,
	};
	let class = if props.correct {
		"confusion-matrix-comparison-item-correct-wrapper"
	} else {
		"confusion-matrix-comparison-item-incorrect-wrapper"
	};
	let class = classes!("confusion-matrix-comparison-item-wrapper", class);
	let value_a = format_option_percent(props.value_a);
	let value_b = format_option_percent(props.value_b);
	html! {
		<div class={class} style={wrapper_style} data-area={props.area}>
			<div class="confusion-matrix-comparison-item-title">{props.label}</div>
			<div class="confusion-matrix-comparison-number-comparison-wrapper">
				<div class="confusion-matrix-comparison-item-value" data-field="value-a">
					{value_a}
				</div>
				<div class="confusion-matrix-comparison-item-value" data-field="value-b">
					{value_b}
				</div>
				<div>
					<Token color?={Some(props.color_a)}>{props.value_a_title}</Token>
				</div>
				<div>
					<Token color?={Some(props.color_b)}>{props.value_b_title}</Token>
				</div>
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct ConfusionMatrixLabelProps {
	pub area: String,
	pub left: Option<bool>,
}

#[component]
pub fn ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
	let left = props.left.unwrap_or(false);
	let justify_items = if left { "end" } else { "center" };
	let style = style! {
		"grid-area" => props.area,
		"justify-items" => justify_items,
	};
	html! {
		<div class="confusion-matrix-comparison-label" style={style}>
			{children}
		</div>
	}
}

pub fn update_confusion_matrix_comparison_item(id: &str, area: &str, value_a: f32, value_b: f32) {
	let document = window().unwrap().document().unwrap();
	let value_a_element = document
		.query_selector(&format!(
			"#{} [data-area='{}'] [data-field='value-a']",
			id, area
		))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	let value_b_element = document
		.query_selector(&format!(
			"#{} [data-area='{}'] [data-field='value-b']",
			id, area
		))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	value_a_element.set_inner_html(&format_percent(value_a));
	value_b_element.set_inner_html(&format_percent(value_b));
}

pub fn update_confusion_matrix_comparison(
	id: &str,
	value_a: ConfusionMatrixComparisonValue,
	value_b: ConfusionMatrixComparisonValue,
) {
	update_confusion_matrix_comparison_item(
		id,
		"false-positive",
		value_a.false_positive,
		value_b.false_positive,
	);
	update_confusion_matrix_comparison_item(
		id,
		"false-negative",
		value_a.false_negative,
		value_b.false_negative,
	);
	update_confusion_matrix_comparison_item(
		id,
		"true-positive",
		value_a.true_positive,
		value_b.true_positive,
	);
	update_confusion_matrix_comparison_item(
		id,
		"true-negative",
		value_a.true_negative,
		value_b.true_negative,
	);
}
