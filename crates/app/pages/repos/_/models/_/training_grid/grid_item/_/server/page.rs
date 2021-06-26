use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub id: String,
	pub model_grid_item_identifier: String,
	pub model_layout_info: ModelLayoutInfo,
	pub model_hyperparameters: Vec<(String, String)>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child("Hyperparameters"))
						.child(ModelHyperparametersTable {
							hyperparameters: self.model_hyperparameters,
						}),
				),
			)
			.into_node()
	}
}

pub struct ModelHyperparametersTable {
	hyperparameters: Vec<(String, String)>,
}

impl Component for ModelHyperparametersTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.children(self.hyperparameters.into_iter().map(
				|(hyperparam_name, hyperparam_value)| {
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child(hyperparam_name))
						.child(
							ui::TableCell::new()
								.width("100%".to_owned())
								.child(hyperparam_value),
						)
				},
			))
			.into_node()
	}
}
