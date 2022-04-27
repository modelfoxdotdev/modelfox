use modelfox_app_date_window::DateWindow;
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct DateWindowSelectField {
	pub date_window: DateWindow,
}

impl DateWindowSelectField {
	pub fn new(date_window: DateWindow) -> DateWindowSelectField {
		DateWindowSelectField { date_window }
	}
}

impl Component for DateWindowSelectField {
	fn into_node(self) -> Node {
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
		ui::SelectField::new()
			.id("date_window_select_field".to_owned())
			.label("Date Window".to_owned())
			.name("date_window".to_owned())
			.options(options)
			.value(self.date_window.to_string())
			.into_node()
	}
}
