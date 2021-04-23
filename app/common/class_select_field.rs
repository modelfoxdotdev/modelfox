use html::{component, html, Props};
use tangram_ui as ui;

#[derive(Props)]
pub struct ClassSelectFieldProps {
	pub class: String,
	pub classes: Vec<String>,
}

#[component]
pub fn ClassSelectField(props: ClassSelectFieldProps) {
	let options = props
		.classes
		.iter()
		.map(|class_name| ui::SelectFieldOption {
			text: class_name.clone(),
			value: class_name.clone(),
		})
		.collect::<Vec<_>>();
	html! {
		<ui::SelectField
			id?="class_select_field"
			label?="Select Class"
			name?="class"
			options?={Some(options)}
			value?={Some(props.class)}
		/>
	}
}
