use super::FieldLabel;
use html::{component, html, Props};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

#[derive(Props)]
pub struct SelectFieldProps {
	#[optional]
	pub disabled: Option<bool>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub label: Option<String>,
	#[optional]
	pub name: Option<String>,
	#[optional]
	pub options: Option<Vec<SelectFieldOption>>,
	#[optional]
	pub placeholder: Option<String>,
	#[optional]
	pub required: Option<bool>,
	#[optional]
	pub value: Option<String>,
}

pub struct SelectFieldOption {
	pub text: String,
	pub value: String,
}

#[component]
pub fn SelectField(props: SelectFieldProps) {
	let options = props.options.unwrap_or_else(Vec::new);
	let value = props.value;
	let autocomplete = value.as_ref().map(|_| "off".to_owned());
	html! {
		<FieldLabel html_for={None}>
			{props.label}
			<select
				autocomplete={autocomplete}
				class="form-select-field"
				disabled={props.disabled}
				id={props.id}
				name={props.name}
				placeholder={props.placeholder}
				required={props.required}
			>
				{options.iter().map(|option| {
					let selected = value
						.as_ref()
						.map(|value| *value == option.value)
						.unwrap_or(false);
					html! {
						<option value={option.value.clone()} selected={selected}>
							{option.text.clone()}
						</option>
					}
				}).collect::<Vec<_>>()}
			</select>
		</FieldLabel>
	}
}

pub fn select_field_submit_on_change(id: String) {
	let document = window().unwrap().document().unwrap();
	let select_element = document.get_element_by_id(&id).unwrap();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: Event| {
		if let Some(event) = event.current_target() {
			let form = event
				.dyn_ref::<HtmlElement>()
				.unwrap()
				.closest("form")
				.unwrap();
			form.unwrap()
				.dyn_ref::<HtmlFormElement>()
				.unwrap()
				.submit()
				.ok();
		}
	}));
	if let Some(select_element) = select_element.dyn_ref::<HtmlSelectElement>() {
		select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
