use html::{classes, component, html, style, Props};

pub enum WindowShade {
	Code,
	Default,
}

#[derive(Props)]
pub struct WindowProps {
	pub padding: Option<bool>,
}

#[component]
pub fn Window(props: WindowProps) {
	let red_style = style! {
		"background-color" => "var(--red)",
	};
	let yellow_style = style! {
		"background-color"=> "var(--yellow)",
	};
	let green_style = style! {
		"background-color"=> "var(--green)",
	};
	let window_body_class = classes! {
		"window-body",
		if props.padding.unwrap_or(false) { Some("window-body-padding") } else { None },
	};
	html! {
		<div class="window-wrapper">
			<div class="window-topbar">
				<div class="window-topbar-button" style={red_style}></div>
				<div class="window-topbar-button" style={yellow_style}></div>
				<div class="window-topbar-button" style={green_style}></div>
			</div>
			<div class={window_body_class}>{children}</div>
		</div>
	}
}
