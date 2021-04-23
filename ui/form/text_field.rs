use super::FieldLabel;
use html::{component, html, Props};

#[derive(Props)]
pub struct TextFieldProps {
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

#[component]
pub fn TextField(props: TextFieldProps) {
	html! {
		<FieldLabel html_for={None}>
			{props.label}
			<input
				id={props.id}
				autocomplete={props.autocomplete}
				class="form-text-field"
				disabled={props.disabled}
				name={props.name}
				placeholder={props.placeholder}
				readonly={props.readonly}
				required={props.required}
				spellcheck={false}
				value={props.value}
			/>
		</FieldLabel>
	}
}
