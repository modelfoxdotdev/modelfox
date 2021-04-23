use crate::date_window::DateWindow;
use html::{component, html, Props};
use tangram_ui as ui;

#[derive(Props)]
pub struct DateWindowSelectFieldProps {
	pub date_window: DateWindow,
}

#[component]
pub fn DateWindowSelectField(props: DateWindowSelectFieldProps) {
	let options = vec![
		ui::SelectFieldOption {
			text: "Today".to_owned(),
			value: "today".to_owned(),
		},
		ui::SelectFieldOption {
			text: "This Month".to_owned(),
			value: "this_month".to_owned(),
		},
		ui::SelectFieldOption {
			text: "This Year".to_owned(),
			value: "this_year".to_owned(),
		},
	];
	html! {
		<ui::SelectField
			id?="date_window_select_field"
			label?="Date Window"
			name?="date_window"
			options?={Some(options)}
			value?={Some(props.date_window.to_string())}
		/>
	}
}
