pub use crate::{enum_column::*, number_column::*, text_column::*};
use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::{date_window::DateWindow, date_window_select_field::DateWindowSelectField};
use tangram_ui as ui;

pub struct Page {
	pub id: String,
	pub alert_name: String,
	pub model_layout_info: ModelLayoutInfo,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child(self.alert_name))
						.child(DateWindowSelectForm {
							date_window: self.date_window,
						})
				),
			)
			.into_node()
	}
}
