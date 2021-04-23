use html::{component, html, Props};
use tangram_ui as ui;

pub struct Pagination {
	pub first_offset: Option<usize>,
	pub next_offset: Option<usize>,
	pub previous_offset: Option<usize>,
	pub last_offset: Option<usize>,
}

#[derive(Props)]
pub struct PaginationButtonsProps {
	pub pagination: Pagination,
	pub form_action: Option<String>,
}

#[component]
pub fn PaginationButtons(props: PaginationButtonsProps) {
	html! {
		<div class="offset-pagination-buttons">
			<ui::Form action?={props.form_action.clone()}>
				{props.pagination.first_offset.map(|first_offset| html! {
					<input
						name="offset"
						type="hidden"
						value={first_offset.to_string()}
					/>
				})}
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					disabled?={Some(props.pagination.first_offset.is_none())}
				>
					{"First"}
				</ui::Button>
			</ui::Form>
			<ui::Form action?={props.form_action.clone()}>
				{props.pagination.previous_offset.map(|previous_offset| html! {
					<input
						name="offset"
						type="hidden"
						value={previous_offset.to_string()}
					/>
				})}
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					disabled?={Some(props.pagination.previous_offset.is_none())}
				>
					{"<"}
				</ui::Button>
			</ui::Form>
			<ui::Form action?={props.form_action.clone()}>
				{props.pagination.next_offset.map(|next_offset| html! {
					<input
						name="offset"
						type="hidden"
						value={next_offset.to_string()}
					/>
				})}
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					disabled?={Some(props.pagination.next_offset.is_none())}
				>
					{">"}
				</ui::Button>
			</ui::Form>
			<ui::Form action?={props.form_action.clone()}>
				{props.pagination.last_offset.map(|last_offset| html! {
					<input
						name="offset"
						type="hidden"
						value={last_offset.to_string()}
					/>
				})}
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					disabled?={Some(props.pagination.last_offset.is_none())}
				>
					{"Last"}
				</ui::Button>
			</ui::Form>
		</div>
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
