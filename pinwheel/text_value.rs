use std::borrow::Cow;

pub struct TextValue(pub Cow<'static, str>);

pub trait IntoTextValue {
	fn into_text_value(self) -> TextValue;
}

impl IntoTextValue for Cow<'static, str> {
	fn into_text_value(self) -> TextValue {
		TextValue(self)
	}
}

impl IntoTextValue for String {
	fn into_text_value(self) -> TextValue {
		TextValue(Cow::Owned(self))
	}
}

impl IntoTextValue for &'static str {
	fn into_text_value(self) -> TextValue {
		TextValue(Cow::Borrowed(self))
	}
}
