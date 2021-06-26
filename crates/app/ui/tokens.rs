use crate::column_type::ColumnType;
use pinwheel::prelude::*;
use tangram_ui as ui;

pub struct UnknownColumnToken;

impl Component for UnknownColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color(ui::colors::GRAY.to_owned())
			.child("Unknown")
			.into_node()
	}
}

pub struct NumberColumnToken;

impl Component for NumberColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color(ui::colors::TEAL.to_owned())
			.child("Number")
			.into_node()
	}
}

pub struct EnumColumnToken;

impl Component for EnumColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color(ui::colors::PURPLE.to_owned())
			.child("Enum")
			.into_node()
	}
}

pub struct TextColumnToken;

impl Component for TextColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color(ui::colors::ORANGE.to_owned())
			.child("Text")
			.into_node()
	}
}

pub struct ColumnTypeToken {
	column_type: ColumnType,
}

impl ColumnTypeToken {
	pub fn new(column_type: ColumnType) -> ColumnTypeToken {
		ColumnTypeToken { column_type }
	}
}

impl Component for ColumnTypeToken {
	fn into_node(self) -> Node {
		match self.column_type {
			ColumnType::Unknown => UnknownColumnToken.into_node(),
			ColumnType::Number => NumberColumnToken.into_node(),
			ColumnType::Enum => EnumColumnToken.into_node(),
			ColumnType::Text => TextColumnToken.into_node(),
		}
	}
}
