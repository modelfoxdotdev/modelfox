use super::FieldLabel;
use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct TextField {
	#[optional]
	pub autocomplete: Option<String>,
	#[optional]
	pub id: Option<String>,
	#[optional]
	pub disabled: Option<bool>,
	#[optional]
	pub label: Option<String>,
	#[optional]
	pub name: Option<String>,
	#[optional]
	pub placeholder: Option<String>,
	#[optional]
	pub readonly: Option<bool>,
	#[optional]
	pub required: Option<bool>,
	#[optional]
	pub value: Option<String>,
}

impl Component for TextField {
	fn into_node(self) -> Node {
		FieldLabel::new(None)
			.child(self.label)
			.child(
				input()
					.attribute("id", self.id)
					.attribute("autocomplete", self.autocomplete)
					.class("form-text-field")
					.attribute("disabled", self.disabled)
					.attribute("name", self.name)
					.attribute("placeholder", self.placeholder)
					.attribute("readonly", self.readonly)
					.attribute("required", self.required)
					.attribute("spellcheck", false)
					.attribute("value", self.value),
			)
			.into_node()
	}
}
