use pinwheel::prelude::*;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct ClassSelectField {
	pub class: String,
	pub classes: Vec<String>,
}

impl Component for ClassSelectField {
	fn into_node(self) -> Node {
		let options = self
			.classes
			.iter()
			.map(|class_name| ui::SelectFieldOption {
				text: class_name.clone(),
				value: class_name.clone(),
			})
			.collect::<Vec<_>>();
		ui::SelectField::new()
			.id("class_select_field".to_owned())
			.label("Select Class".to_owned())
			.name("class".to_owned())
			.options(Some(options))
			.value(Some(self.class))
			.into_node()
	}
}
