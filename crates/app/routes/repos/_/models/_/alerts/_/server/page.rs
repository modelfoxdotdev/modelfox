use pinwheel::prelude::*;
use tangram_app_core::alerts::{AlertData, AlertModelType};
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub alert: AlertData,
	pub alert_id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub model_type: AlertModelType,
	pub error: Option<String>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child(format!("Alert {}", self.alert_id)))
						.child(ui::P::new().child(format!("{:?}", self.alert))),
				),
			)
			.into_node()
	}
}
