use crate::Card;
use html::{component, html, Props};
use wasm_bindgen::JsCast;
use web_sys::*;

#[derive(Props)]
pub struct NumberCardProps {
	#[optional]
	pub id: Option<String>,
	pub title: String,
	pub value: String,
}

#[component]
pub fn NumberCard(props: NumberCardProps) {
	html! {
		<Card>
			<div id={props.id} class="number-wrapper">
				<div class="number-value">{props.value}</div>
				<div class="number-title">{props.title}</div>
			</div>
		</Card>
	}
}

pub fn update_number(id: &str, value: String) {
	let document = window().unwrap().document().unwrap();
	let value_element = document
		.query_selector(&format!("#{} .number-value", id))
		.unwrap()
		.unwrap()
		.dyn_into::<HtmlElement>()
		.unwrap();
	value_element.set_inner_html(&value);
}
