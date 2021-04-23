use html::{component, html, style, Props};

#[derive(Props)]
pub struct ButtonProps {
	#[optional]
	pub button_type: Option<ButtonType>,
	#[optional]
	pub color: Option<String>,
	#[optional]
	pub disabled: Option<bool>,
	#[optional]
	pub download: Option<String>,
	#[optional]
	pub href: Option<String>,
	#[optional]
	pub id: Option<String>,
}

pub enum ButtonType {
	Button,
	Reset,
	Submit,
}

#[component]
pub fn Button(props: ButtonProps) {
	let button_type = props.button_type.unwrap_or(ButtonType::Button);
	let button_type = match button_type {
		ButtonType::Button => "button",
		ButtonType::Reset => "reset",
		ButtonType::Submit => "submit",
	};
	let style = style! {
		"background-color" => props.color,
	};
	if let Some(href) = props.href {
		html! {
			<a
				class="button"
				disabled={props.disabled}
				download={props.download}
				href={href}
				style={style}
			>
				{children}
			</a>
		}
	} else {
		html! {
			<button
				class="button"
				disabled={props.disabled}
				id={props.id}
				style={style}
				type={button_type}
			>
				{children}
			</button>
		}
	}
}
