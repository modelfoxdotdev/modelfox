use crate::date_window::DateWindow;
use pinwheel::prelude::*;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct DateWindowSelectField {
	pub date_window: DateWindow,
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
			.options(Some(options))
			.value(Some(self.date_window.to_string()))
			.into_node()
	}
}
