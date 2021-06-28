use super::FieldLabel;
use pinwheel::prelude::*;

#[derive(builder, Default, new)]
#[new(default)]
pub struct TextField {
	#[builder]
	pub autocomplete: Option<String>,
	#[builder]
	pub id: Option<String>,
	#[builder]
	pub disabled: Option<bool>,
	#[builder]
	pub label: Option<String>,
	#[builder]
	pub name: Option<String>,
	#[builder]
	pub placeholder: Option<String>,
	#[builder]
	pub readonly: Option<bool>,
	#[builder]
	pub required: Option<bool>,
	#[builder]
	pub value: Option<String>,
}

impl Component for TextField {
	fn into_node(self) -> Node {
		FieldLabel::new()
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
