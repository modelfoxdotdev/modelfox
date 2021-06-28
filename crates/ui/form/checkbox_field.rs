use super::FieldLabel;
use pinwheel::prelude::*;

#[derive(builder, Default, new)]
#[new(default)]
pub struct CheckboxField {
	#[builder]
	pub label: Option<String>,
	#[builder]
	pub name: Option<String>,
	#[builder]
	pub placeholder: Option<String>,
	#[builder]
	pub disabled: Option<bool>,
	#[builder]
	pub value: Option<String>,
	#[builder]
	pub checked: Option<bool>,
}

impl Component for CheckboxField {
	fn into_node(self) -> Node {
		FieldLabel::new()
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
