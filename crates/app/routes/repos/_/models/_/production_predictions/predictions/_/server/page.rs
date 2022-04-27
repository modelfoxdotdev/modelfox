use modelfox_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use modelfox_app_ui::predict::PredictOutput;
use modelfox_id::Id;
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Page {
	pub id: Id,
	pub date: String,
	pub identifier: String,
	pub predict_output: PredictOutput,
	pub model_layout_info: ModelLayoutInfo,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("modelfox_app_production_prediction_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new("Prediction"))
						.child(PredictionTable {
							identifier: self.identifier,
							date: self.date,
						})
						.child(self.predict_output),
				),
			)
			.into_node()
	}
}

pub struct PredictionTable {
	pub date: String,
	pub identifier: String,
}

impl Component for PredictionTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Identifier"))
						.child(ui::TableHeaderCell::new().child("Date")),
				),
			)
			.child(
				ui::TableBody::new().child(
					ui::TableRow::new()
						.child(ui::TableCell::new().child(self.identifier))
						.child(ui::TableCell::new().child(self.date)),
				),
			)
			.into_node()
	}
}
