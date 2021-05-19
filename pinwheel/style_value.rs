use std::borrow::Cow;

#[derive(Clone)]
pub struct StyleValue(pub Option<Cow<'static, str>>);

pub trait IntoStyleValue {
	fn into_style_value(self) -> StyleValue;
}

impl IntoStyleValue for Cow<'static, str> {
	fn into_style_value(self) -> StyleValue {
		StyleValue(Some(self))
	}
}

impl IntoStyleValue for String {
	fn into_style_value(self) -> StyleValue {
		StyleValue(Some(Cow::Owned(self)))
	}
}

impl IntoStyleValue for &'static str {
	fn into_style_value(self) -> StyleValue {
		StyleValue(Some(Cow::Borrowed(self)))
	}
}

impl IntoStyleValue for Option<Cow<'static, str>> {
	fn into_style_value(self) -> StyleValue {
		StyleValue(self)
	}
}

impl IntoStyleValue for Option<String> {
	fn into_style_value(self) -> StyleValue {
		StyleValue(self.map(Cow::Owned))
	}
}

impl IntoStyleValue for Option<&'static str> {
	fn into_style_value(self) -> StyleValue {
		StyleValue(self.map(Cow::Borrowed))
	}
}
