use super::FieldLabel;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct CheckboxField {
	#[optional]
	pub label: Option<String>,
	#[optional]
	pub name: Option<String>,
	#[optional]
	pub placeholder: Option<String>,
	#[optional]
	pub disabled: Option<bool>,
	#[optional]
	pub value: Option<String>,
	#[optional]
	pub checked: Option<bool>,
}

impl Component for CheckboxField {
	fn into_node(self) -> Node {
		FieldLabel::new(None)
			.child(self.label)
			.child(
				input()
					.class("form-checkbox-field")
					.attribute("name", self.name)
					.attribute("placeholder", self.placeholder)
					.attribute("disabled", self.disabled)
					.attribute("type", "checkbox")
					.attribute("value", self.value)
					.attribute("checked", self.checked),
			)
			.into_node()
	}
}
