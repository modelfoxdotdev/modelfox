use std::borrow::Cow;

pub struct StringValue(pub Cow<'static, str>);

pub trait IntoStringValue {
	fn into_string_value(self) -> StringValue;
}

impl IntoStringValue for Cow<'static, str> {
	fn into_string_value(self) -> StringValue {
		StringValue(self)
	}
}

impl IntoStringValue for String {
	fn into_string_value(self) -> StringValue {
		StringValue(Cow::Owned(self))
	}
}

impl IntoStringValue for &'static str {
	fn into_string_value(self) -> StringValue {
		StringValue(Cow::Borrowed(self))
	}
}
