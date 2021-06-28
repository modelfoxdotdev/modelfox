use pinwheel::prelude::*;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Button {
	#[builder]
	pub button_type: Option<ButtonType>,
	#[builder]
	pub color: Option<String>,
	#[builder]
	pub disabled: Option<bool>,
	#[builder]
	pub download: Option<String>,
	#[builder]
	pub href: Option<String>,
	#[builder]
	pub open_new_window: bool,
	#[builder]
	pub id: Option<String>,
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
			let target = if self.open_new_window {
				Some("_blank")
			} else {
				None
			};
			a().class("button")
				.download(self.download)
				.href(href)
				.target(target)
				.style(style::BACKGROUND_COLOR, self.color)
				.child(self.children)
				.into_node()
		} else {
			button()
				.class("button")
				.disabled(self.disabled)
				.id(self.id)
				.style(style::BACKGROUND_COLOR, self.color)
				.attribute("type", button_type)
				.child(self.children)
				.into_node()
		}
	}
}
