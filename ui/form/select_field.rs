use super::FieldLabel;
use pinwheel::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

#[derive(ComponentBuilder)]
pub struct SelectField {
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

impl Component for SelectField {
	fn into_node(self) -> Node {
		let options = self.options.unwrap_or_else(Vec::new);
		let value = self.value;
		let autocomplete = value.as_ref().map(|_| "off".to_owned());
		FieldLabel::new(None)
			.child(self.label)
			.child(
				select()
					.attribute("autocomplete", autocomplete)
					.class("form-select-field")
					.attribute("disabled", self.disabled)
					.attribute("id", self.id)
					.attribute("name", self.name)
					.attribute("placeholder", self.placeholder)
					.attribute("required", self.required)
					.children(options.iter().map(|option| {
						let selected = value
							.as_ref()
							.map(|value| *value == option.value)
							.unwrap_or(false);
						html::option()
							.attribute("value", option.value.clone())
							.attribute("selected", selected)
							.child(option.text.clone())
					})),
			)
			.into_node()
	}
}

pub fn select_field_submit_on_change(id: String) {
	let document = dom::window().unwrap().document().unwrap();
	let select_element = document.get_element_by_id(&id).unwrap();
	let callback_fn = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: dom::Event| {
		if let Some(event) = event.current_target() {
			let form = event
				.dyn_ref::<dom::HtmlElement>()
				.unwrap()
				.closest("form")
				.unwrap();
			form.unwrap()
				.dyn_ref::<dom::HtmlFormElement>()
				.unwrap()
				.submit()
				.ok();
		}
	}));
	if let Some(select_element) = select_element.dyn_ref::<dom::HtmlSelectElement>() {
		select_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
	}
	callback_fn.forget();
}
