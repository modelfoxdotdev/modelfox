use html::{component, html, style, Props};

#[derive(Props)]
pub struct TokenProps {
	#[optional]
	pub color: Option<String>,
}

#[component]
pub fn Token(props: TokenProps) {
	let style = style! {
		"background-color" => props.color,
	};
	html! {
		<span class="token" style={style}>
			{children}
		</span>
	}
}
