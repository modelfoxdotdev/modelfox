use super::FieldLabel;
use pinwheel::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

#[derive(ComponentBuilder)]
pub struct FileField {
	#[optional]
	pub disabled: Option<bool>,
	#[optional]
	pub label: Option<String>,
	#[optional]
	pub name: Option<String>,
	#[optional]
	pub required: Option<bool>,
}

impl Component for FileField {
	fn into_node(self) -> Node {
		FieldLabel::new(None)
			.child(self.label)
			.child(
				div().class("form-file-wrapper").child("Choose File").child(
					input()
						.class("form-file-input")
						.attribute("name", self.name)
						.attribute("required", self.required)
						.attribute("disabled", self.disabled)
						.attribute("type", "file"),
				),
			)
			.into_node()
	}
}

/** When using a custom 'Choose File' prompt, it is necessary to use JS to update it to show the selected file name. */
pub fn boot_file_fields() {
	let document = dom::window().unwrap().document().unwrap();
	let file_input_elements = document.query_selector_all("input[type=file]").unwrap();
	for file_input_element_index in 0..file_input_elements.length() {
		let file_input_element = file_input_elements.item(file_input_element_index).unwrap();
		update_file_input_element(&file_input_element.clone());
		let file_input_element_for_closure = file_input_element.clone();
		let callback_fn = Closure::<dyn FnMut()>::wrap(Box::new(move || {
			update_file_input_element(&file_input_element_for_closure)
		}));
		file_input_element
			.add_event_listener_with_callback("change", callback_fn.as_ref().unchecked_ref())
			.unwrap();
		callback_fn.forget();
		fn update_file_input_element(file_input_element: &dom::EventTarget) {
			let file = file_input_element
				.dyn_ref::<dom::HtmlInputElement>()
				.unwrap()
				.files()
				.and_then(|files| files.item(0));
			if let Some(file) = file {
				let file_name = file.name();
				if let Some(file_input_element) = file_input_element
					.dyn_ref::<dom::HtmlInputElement>()
					.and_then(|element| element.parent_element())
				{
					file_input_element
						.first_child()
						.unwrap()
						.set_text_content(Some(&file_name));
				};
			}
		}
	}
}
