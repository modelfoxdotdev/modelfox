use pinwheel::prelude::*;
use tangram_ui as ui;

pub struct Pagination {
	pub first_offset: Option<usize>,
	pub next_offset: Option<usize>,
	pub previous_offset: Option<usize>,
	pub last_offset: Option<usize>,
}

#[derive(ComponentBuilder)]
pub struct PaginationButtons {
	pub pagination: Pagination,
	pub form_action: Option<String>,
}

impl Component for PaginationButtons {
	fn into_node(self) -> Node {
		div()
			.class("offset-pagination-buttons")
			.child(
				ui::Form::new()
					.action(self.form_action.clone())
					.child(self.pagination.first_offset.map(|first_offset| {
						input()
							.name("offset")
							.attribute("type", "hidden")
							.value(first_offset.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(Some(ui::ButtonType::Submit))
							.disabled(Some(self.pagination.first_offset.is_none()))
							.child("First"),
					),
			)
			.child(
				ui::Form::new()
					.action(self.form_action.clone())
					.child(self.pagination.previous_offset.map(|previous_offset| {
						input()
							.attribute("name", "offset")
							.attribute("type", "hidden")
							.attribute("value", previous_offset.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(Some(ui::ButtonType::Submit))
							.disabled(Some(self.pagination.previous_offset.is_none()))
							.child("<"),
					),
			)
			.child(
				ui::Form::new()
					.action(self.form_action.clone())
					.child(self.pagination.next_offset.map(|next_offset| {
						input()
							.attribute("name", "offset")
							.attribute("type", "hidden")
							.attribute("value", next_offset.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(Some(ui::ButtonType::Submit))
							.disabled(Some(self.pagination.next_offset.is_none()))
							.child(">"),
					),
			)
			.child(
				ui::Form::new()
					.action(self.form_action.clone())
					.child(self.pagination.last_offset.map(|last_offset| {
						input()
							.attribute("name", "offset")
							.attribute("type", "hidden")
							.attribute("value", last_offset.to_string())
					}))
					.child(
						ui::Button::new()
							.button_type(Some(ui::ButtonType::Submit))
							.disabled(Some(self.pagination.last_offset.is_none()))
							.child("Last"),
					),
			)
			.into_node()
	}
}
pub fn compute_pagination(offset: usize, n_items: usize, limit: usize) -> Pagination {
	let first_offset = if offset == 0 { None } else { Some(0) };
	let next_offset = if offset + limit < n_items {
		Some(offset + limit)
	} else {
		None
	};
	let previous_offset = if offset == 0 {
		None
	} else if offset < limit {
		Some(0)
	} else {
		Some(offset - limit)
	};
	let last_offset = n_items - limit;
	let last_offset = if offset >= last_offset {
		None
	} else {
		Some(last_offset)
	};
	Pagination {
		first_offset,
		next_offset,
		previous_offset,
		last_offset,
	}
}
