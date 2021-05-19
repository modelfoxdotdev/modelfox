use pinwheel::prelude::*;
use tangram_app_common::predict::PredictOutput;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub identifier: String,
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	NotFound(Box<NotFound>),
	Found(Box<Found>),
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::NotFound(inner) => (*inner).into_node(),
			Inner::Found(inner) => (*inner).into_node(),
		};
		Document::new()
			.client("tangram_app_production_prediction_client")
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child("Prediction"))
						.child(inner),
				),
			)
			.into_node()
	}
}

pub struct NotFound {
	pub identifier: String,
}

impl Component for NotFound {
	fn into_node(self) -> Node {
		ui::Alert::new(ui::Level::Danger)
			.child("Prediction with identifier ")
			.child(b().child(self.identifier))
			.child(" not found.")
			.into_node()
	}
}

pub struct Found {
	pub date: String,
	pub identifier: String,
	pub predict_output: PredictOutput,
}

impl Component for Found {
	fn into_node(self) -> Node {
		fragment()
			.child(PredictionTable::new(self.identifier, self.date))
			.child(self.predict_output)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
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
