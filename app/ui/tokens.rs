use crate::column_type::ColumnType;
use pinwheel::prelude::*;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct UnknownColumnToken {
	#[children]
	pub children: Vec<Node>,
}

impl Component for UnknownColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color("var(--gray)".to_owned())
			.child("Unknown")
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct NumberColumnToken {
	#[children]
	pub children: Vec<Node>,
}

impl Component for NumberColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color("var(--teal)".to_owned())
			.child("Number")
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct EnumColumnToken {
	#[children]
	pub children: Vec<Node>,
}

impl Component for EnumColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color("var(--purple)".to_owned())
			.child("Enum")
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TextColumnToken {
	#[children]
	pub children: Vec<Node>,
}

impl Component for TextColumnToken {
	fn into_node(self) -> Node {
		ui::Token::new()
			.color("var(--orange)".to_owned())
			.child("Text")
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ColumnTypeToken {
	column_type: ColumnType,
}

impl Component for ColumnTypeToken {
	fn into_node(self) -> Node {
		match self.column_type {
			ColumnType::Unknown => UnknownColumnToken::new().into_node(),
			ColumnType::Number => NumberColumnToken::new().into_node(),
			ColumnType::Enum => EnumColumnToken::new().into_node(),
			ColumnType::Text => TextColumnToken::new().into_node(),
		}
	}
}
