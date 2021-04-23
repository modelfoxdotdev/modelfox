use html::{component, html, Props};

#[derive(Props)]
pub struct FieldLabelProps {
	pub html_for: Option<String>,
}

#[component]
pub fn FieldLabel(props: FieldLabelProps) {
	html! {
		<label class="field-label" for={props.html_for}>
			{children}
		</label>
	}
}
