use html::{classes, component, html, style, Props};

#[derive(Props)]
pub struct TableProps {
	#[optional]
	pub width: Option<String>,
}

#[component]
pub fn Table(props: TableProps) {
	let style = style! {
		"width" => props.width.unwrap_or_else(|| "auto".into()),
	};
	html! {
		<div class="table-wrapper" >
			<table class="table" style={style}>
				{children}
			</table>
		</div>
	}
}

#[component]
pub fn TableHeader() {
	html! {
		<thead class="table-header">
			{children}
		</thead>
	}
}

#[component]
pub fn TableBody() {
	html! {
		<tbody>{children}</tbody>
	}
}

#[derive(Props)]
pub struct TableRowProps {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub text_color: Option<String>,
}

#[component]
pub fn TableRow(props: TableRowProps) {
	let style = style! {
		"background-color" => props.color,
		"color" => props.text_color,
	};
	html! {
		<tr style={style}>
			{children}
		</tr>
	}
}

#[derive(Props)]
pub struct TableHeaderCellProps {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub expand: Option<bool>,
	#[optional]
	pub text_align: Option<TextAlign>,
}

pub enum TextAlign {
	Left,
	Center,
	Right,
}

#[component]
pub fn TableHeaderCell(props: TableHeaderCellProps) {
	let style = style! {
		"background-color" => props.color,
	};
	let text_align_class = props
		.text_align
		.map(|text_align| match text_align {
			TextAlign::Left => "table-align-left",
			TextAlign::Right => "table-align-right",
			TextAlign::Center => "table-align-center",
		})
		.unwrap_or("table-align-left");
	let expand = props
		.expand
		.and_then(|expand| if expand { Some("table-expand") } else { None });
	let class = classes!("table-header-cell", text_align_class, expand);
	html! {
		<th class={class} style={style}>
			{children}
		</th>
	}
}

#[derive(Props)]
pub struct TableCellProps {
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub expand: Option<bool>,
}

#[component]
pub fn TableCell(props: TableCellProps) {
	let style = style! {
		"background-color" => props.color,
	};
	let expand = props
		.expand
		.and_then(|expand| if expand { Some("table-expand") } else { None });
	let class = classes!("table-cell", expand);
	html! {
		<td class={class} style={style}>
			{children}
		</td>
	}
}
