use super::FieldLabel;
use pinwheel::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

#[derive(builder, Default, new)]
#[new(default)]
pub struct SelectField {
	#[builder]
	pub disabled: Option<bool>,
	#[builder]
	pub id: Option<String>,
	#[builder]
	pub class: Option<String>,
	#[builder]
	pub label: Option<String>,
	#[builder]
	pub name: Option<String>,
	#[builder]
	pub options: Option<Vec<SelectFieldOption>>,
	#[builder]
	pub placeholder: Option<String>,
	#[builder]
	pub required: Option<bool>,
	#[builder]
	pub value: Option<String>,
	#[builder]
	pub on_change: Option<Box<dyn Fn(String)>>,
}

#[derive(Clone)]
pub struct SelectFieldOption {
	pub text: String,
	pub value: String,
}

impl Component for SelectField {
	fn into_node(self) -> Node {
		let options = self.options.unwrap_or_else(Vec::new);
		let value = self.value;
		let autocomplete = value.as_ref().map(|_| "off".to_owned());
		let oninput = {
			let on_change = self.on_change;
			move |event: dom::InputEvent| {
				let current_target = event
					.current_target()
					.unwrap()
					.dyn_into::<dom::HtmlSelectElement>()
					.unwrap();
				let value = current_target.value();
				if let Some(on_change) = on_change.as_ref() {
					on_change(value);
				}
			}
		};
		let class = if let Some(class) = self.class {
			format!("form-select-field {}", class)
		} else {
			"form-select-field".to_owned()
		};
		FieldLabel::new()
			.child(self.label)
			.child(
				select()
					.attribute("autocomplete", autocomplete)
					.class(class)
					.attribute("disabled", self.disabled)
					.attribute("id", self.id)
					.attribute("name", self.name)
					.attribute("placeholder", self.placeholder)
					.attribute("required", self.required)
					.oninput(oninput)
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
