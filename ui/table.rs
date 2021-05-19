use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Table {
	#[optional]
	pub width: Option<String>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for Table {
	fn into_node(self) -> Node {
		let table = table()
			.style(
				style::WIDTH,
				self.width.unwrap_or_else(|| "auto".to_owned()),
			)
			.child(self.children);
		div().class("table").child(table).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TableHeader {
	#[children]
	pub children: Vec<Node>,
}

impl Component for TableHeader {
	fn into_node(self) -> Node {
		thead().child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TableBody {
	#[children]
	pub children: Vec<Node>,
}

impl Component for TableBody {
	fn into_node(self) -> Node {
		tbody().child(self.children).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TableRow {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub text_color: Option<String>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for TableRow {
	fn into_node(self) -> Node {
		tr().style(style::BACKGROUND_COLOR, self.color)
			.style(style::COLOR, self.text_color)
			.child(self.children)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TableHeaderCell {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub expand: Option<bool>,
	#[optional]
	pub align: Option<Align>,
	#[children]
	pub children: Vec<Node>,
}

pub enum Align {
	Left,
	Center,
	Right,
}

impl Component for TableHeaderCell {
	fn into_node(self) -> Node {
		let text_align = self.align.map(|text_align| match text_align {
			Align::Left => "left",
			Align::Center => "center",
			Align::Right => "right",
		});
		let width = self
			.expand
			.and_then(|expand| if expand { Some("100%") } else { None });
		th().style(style::TEXT_ALIGN, text_align)
			.style(style::WIDTH, width)
			.style(style::BACKGROUND_COLOR, self.color)
			.child(self.children)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TableCell {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub expand: Option<bool>,
	#[optional]
	pub align: Option<Align>,
	#[children]
	pub children: Vec<Node>,
}

impl Component for TableCell {
	fn into_node(self) -> Node {
		let text_align = self.align.map(|text_align| match text_align {
			Align::Left => "left",
			Align::Center => "center",
			Align::Right => "right",
		});
		let width = self
			.expand
			.and_then(|expand| if expand { Some("100%") } else { None });
		td().style(style::TEXT_ALIGN, text_align)
			.style(style::WIDTH, width)
			.style(style::BACKGROUND_COLOR, self.color)
			.child(self.children)
			.into_node()
	}
}
