use pinwheel::prelude::*;

#[derive(ComponentBuilder)]
pub struct Button {
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
	#[children]
	pub children: Vec<Node>,
}

pub enum ButtonType {
	Button,
	Reset,
	Submit,
}

impl Component for Button {
	fn into_node(self) -> Node {
		let button_type = self.button_type.unwrap_or(ButtonType::Button);
		let button_type = match button_type {
			ButtonType::Button => "button",
			ButtonType::Reset => "reset",
			ButtonType::Submit => "submit",
		};
		if let Some(href) = self.href {
			a().class("button")
				.download(self.download)
				.href(href)
				.style(
					style::BACKGROUND_COLOR,
					self.color.unwrap_or_else(|| "".to_owned()),
				)
				.child(self.children)
				.into_node()
		} else {
			button()
				.class("button")
				.disabled(self.disabled)
				.id(self.id)
				.style(
					style::BACKGROUND_COLOR,
					self.color.unwrap_or_else(|| "".to_owned()),
				)
				.attribute("type", button_type)
				.child(self.children)
				.into_node()
		}
	}
}
