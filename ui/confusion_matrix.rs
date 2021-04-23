use crate::Token;
use html::{classes, component, html, style, Props};
use num::ToPrimitive;
use tangram_number_formatter::PercentFormatter;

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

#[derive(Props)]
pub struct ConfusionMatrixProps {
	pub class_label: String,
	pub false_negatives: Option<usize>,
	pub false_positives: Option<usize>,
	pub true_negatives: Option<usize>,
	pub true_positives: Option<usize>,
}

#[component]
pub fn ConfusionMatrix(props: ConfusionMatrixProps) {
	let total = if let (
		Some(false_negatives),
		Some(false_positives),
		Some(true_negatives),
		Some(true_positives),
	) = (
		props.false_negatives,
		props.false_positives,
		props.true_negatives,
		props.true_positives,
	) {
		Some(true_positives + true_negatives + false_positives + false_negatives)
	} else {
		None
	};
	html! {
		<div class="confusion-matrix-wrapper">
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
			<ConfusionMatrixItem
				area="true-positive"
				correct={true}
				title="True Positives"
				total={total}
				value={props.true_positives}
			/>
			<ConfusionMatrixItem
				area="false-positive"
				title="False Positives"
				correct={false}
				total={total}
				value={props.false_positives}
			/>
			<ConfusionMatrixItem
				area="false-negative"
				title="False Negatives"
				correct={false}
				total={total}
				value={props.false_negatives}
			/>
			<ConfusionMatrixItem
				area="true-negative"
				correct={true}
				title="True Negatives"
				total={total}
				value={props.true_negatives}
			/>
		</div>
	}
}

#[derive(Props)]
pub struct ConfusionMatrixItemProps {
	area: String,
	correct: bool,
	title: String,
	total: Option<usize>,
	value: Option<usize>,
}

#[component]
fn ConfusionMatrixItem(props: ConfusionMatrixItemProps) {
	let item_wrapper_style = style! {
		"grid-area" => props.area,
	};
	let class = match props.correct {
		true => "confusion-matrix-item-correct-wrapper",
		false => "confusion-matrix-item-incorrect-wrapper",
	};
	let class = classes!("confusion-matrix-item-wrapper", class);
	let percent = if let (Some(value), Some(total)) = (props.value, props.total) {
		Some(value.to_f32().unwrap() / total.to_f32().unwrap())
	} else {
		None
	};
	let value = props
		.value
		.map(|value| value.to_string())
		.unwrap_or_else(|| "N/A".to_owned());
	let percent = PercentFormatter::default().format_option(percent);
	html! {
		<div
			class={class}
			style={item_wrapper_style}
		>
			<div class="confusion-matrix-item-title">
				{props.title}
			</div>
			<div class="confusion-matrix-item-value">
				{value}
			</div>
			<div class="confusion-matrix-item-percent">
				{percent}
			</div>
		</div>
	}
}

#[derive(Props)]
pub struct ConfusionMatrixLabelProps {
	area: String,
	left: Option<bool>,
}

#[component]
fn ConfusionMatrixLabel(props: ConfusionMatrixLabelProps) {
	let left = props.left.unwrap_or(false);
	let justify_items = if left { "end" } else { "auto" };
	let style = style! {
		"grid-area" => props.area,
		"justify-items" => justify_items,
	};
	html! {
		<div class="confusion-matrix-label" style={style}>
			{children}
		</div>
	}
}
