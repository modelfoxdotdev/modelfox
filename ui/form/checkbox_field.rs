use super::FieldLabel;
use html::{component, html, Props};

#[derive(Props)]
pub struct CheckboxFieldProps {
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
}

#[component]
pub fn CheckboxField(props: CheckboxFieldProps) {
	html! {
		<FieldLabel html_for={None}>
			{props.label}
			<input
				class="form-checkbox-field"
				name={props.name}
				placeholder={props.placeholder}
				disabled={props.disabled}
				type="checkbox"
				value={props.value}
			/>
		</FieldLabel>
	}
}
