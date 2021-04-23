use crate::column_type::ColumnType;
use html::{component, html};
use tangram_ui as ui;

pub const TRAINING_COLOR: &str = ui::colors::BLUE;
pub const PRODUCTION_COLOR: &str = ui::colors::GREEN;
pub const BASELINE_COLOR: &str = ui::colors::GRAY;
pub const SELECTED_THRESHOLD_COLOR: &str = ui::colors::BLUE;

#[component]
pub fn UnknownColumnToken() {
	html! {
		<ui::Token color?="var(--gray)">
			{"Unknown"}
		</ui::Token>
	}
}

#[component]
pub fn NumberColumnToken() {
	html! {
		<ui::Token color?="var(--teal)">
			{"Number"}
		</ui::Token>
	}
}

#[component]
pub fn EnumColumnToken() {
	html! {
		<ui::Token color?="var(--purple)">
			{"Enum"}
		</ui::Token>
	}
}

#[component]
pub fn TextColumnToken() {
	html! {
		<ui::Token color?="var(--orange)">
			{"Text"}
		</ui::Token>
	}
}

pub fn column_type_token(column_type: &ColumnType) -> html::Node {
	match column_type {
		ColumnType::Unknown => html! {
			<UnknownColumnToken />
		},
		ColumnType::Number => html! {
			<NumberColumnToken />
		},
		ColumnType::Enum => html! {
			<EnumColumnToken />
		},
		ColumnType::Text => html! {
			<TextColumnToken />
		},
	}
}
