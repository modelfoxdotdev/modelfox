use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_id::Id;
use tangram_ui as ui;

pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub pagination: Pagination,
	pub prediction_table: Option<PredictionTable>,
}

pub struct PredictionTable {
	pub rows: Vec<PredictionTableRow>,
}

pub struct PredictionTableRow {
	pub id: Id,
	pub date: String,
	pub identifier: String,
	pub output: String,
}

pub struct Pagination {
	pub after: Option<usize>,
	pub before: Option<usize>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let table = self.prediction_table.as_ref().map(|prediction_table| {
			ui::Table::new()
				.width("100%".to_owned())
				.child(
					ui::TableHeader::new().child(
						ui::TableRow::new()
							.child(ui::TableHeaderCell::new().child("Identifier"))
							.child(ui::TableHeaderCell::new().child("Date"))
							.child(ui::TableHeaderCell::new().child("Output")),
					),
				)
				.child(
					ui::TableBody::new().children(prediction_table.rows.iter().map(|prediction| {
						ui::TableRow::new()
							.child(
								ui::TableCell::new().child(
									ui::Link::new()
										.href(format!("./predictions/{}", prediction.id))
										.child(prediction.identifier.clone()),
								),
							)
							.child(ui::TableCell::new().child(prediction.date.clone()))
							.child(ui::TableCell::new().child(prediction.output.clone()))
					})),
				)
		});
		let prev_next_buttons = div()
			.class("pagination-buttons")
			.child(
				ui::Form::new()
					.child(self.pagination.after.map(|after| {
						input()
							.attribute("name", "after")
							.attribute("type", "hidden")
							.attribute("value", after.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.disabled(self.pagination.after.is_none())
							.child("Newer"),
					),
			)
			.child(
				ui::Form::new()
					.child(self.pagination.before.map(|before| {
						input()
							.attribute("name", "before")
							.attribute("type", "hidden")
							.attribute("value", before.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.disabled(self.pagination.before.is_none())
							.child("Older"),
					),
			);
		let predictions = if self.prediction_table.is_none() {
			ui::P::new()
				.child("You have not yet logged any predictions.")
				.into_node()
		} else {
			fragment()
				.child(
					ui::Form::new().post(true).child(
						div()
							.class("search-bar-wrapper")
							.child(
								ui::TextField::new()
									.autocomplete("off".to_owned())
									.label("Identifier".to_owned())
									.name("identifier".to_owned()),
							)
							.child(
								ui::Button::new()
									.button_type(ui::ButtonType::Submit)
									.child("Lookup"),
							),
					),
				)
				.child(table)
				.child(prev_next_buttons)
				.into_node()
		};
		let inner = ui::S1::new()
			.child(ui::H1::new("Production Predictions"))
			.child(predictions);
		Document::new()
			.child(ModelLayout::new(self.model_layout_info).child(inner))
			.into_node()
	}
}
