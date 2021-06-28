pub use self::{checkbox_field::*, field_label::*, file_field::*, select_field::*, text_field::*};
use pinwheel::prelude::*;

mod checkbox_field;
mod field_label;
mod file_field;
mod select_field;
mod text_field;

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct Form {
	#[builder]
	pub action: Option<String>,
	#[builder]
	pub autocomplete: Option<String>,
	#[builder]
	pub enc_type: Option<String>,
	#[builder]
	pub id: Option<String>,
	#[builder]
	pub onsubmit: Option<String>,
	#[builder]
	pub post: Option<bool>,
	pub children: Vec<Node>,
}

impl Component for Form {
	fn into_node(self) -> Node {
		let method = if self.post.unwrap_or(false) {
			Some("post".to_owned())
		} else {
			None
		};
		form()
			.attribute("action", self.action)
			.attribute("autocomplete", self.autocomplete)
			.class("form")
			.attribute("enctype", self.enc_type)
			.attribute("id", self.id)
			.attribute("onsubmit", self.onsubmit)
			.attribute("method", method)
			.child(self.children)
			.into_node()
	}
}
