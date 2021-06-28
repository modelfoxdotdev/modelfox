use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Table {
	#[builder]
	pub width: Option<String>,
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

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TableHeader {
	pub children: Vec<Node>,
}

impl Component for TableHeader {
	fn into_node(self) -> Node {
		thead().child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TableBody {
	pub children: Vec<Node>,
}

impl Component for TableBody {
	fn into_node(self) -> Node {
		tbody().child(self.children).into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TableRow {
	#[builder]
	pub color: Option<String>,
	#[builder]
	pub text_color: Option<String>,
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

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TableHeaderCell {
	#[builder]
	pub color: Option<String>,
	#[builder]
	pub width: Option<String>,
	#[builder]
	pub align: Option<Align>,
	pub children: Vec<Node>,
}

pub enum Align {
	Left,
	Center,
	Right,
}

impl Component for TableHeaderCell {
	fn into_node(self) -> Node {
		let text_align = self
			.align
			.map(|text_align| match text_align {
				Align::Left => "left",
				Align::Center => "center",
				Align::Right => "right",
			})
			.unwrap_or("left");
		th().style(style::TEXT_ALIGN, text_align)
			.style(style::WIDTH, self.width)
			.style(style::BACKGROUND_COLOR, self.color)
			.child(self.children)
			.into_node()
	}
}

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct TableCell {
	#[builder]
	pub color: Option<String>,
	#[builder]
	pub width: Option<String>,
	#[builder]
	pub align: Option<Align>,
	pub children: Vec<Node>,
}

impl Component for TableCell {
	fn into_node(self) -> Node {
		let text_align = self.align.map(|text_align| match text_align {
			Align::Left => "left",
			Align::Center => "center",
			Align::Right => "right",
		});
		td().style(style::TEXT_ALIGN, text_align)
			.style(style::WIDTH, self.width)
			.style(style::BACKGROUND_COLOR, self.color)
			.child(self.children)
			.into_node()
	}
}
