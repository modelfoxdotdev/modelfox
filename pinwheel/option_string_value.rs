use std::borrow::Cow;

#[derive(Clone)]
pub struct OptionStringValue(pub Option<Cow<'static, str>>);

pub trait IntoOptionStringValue {
	fn into_option_string_value(self) -> OptionStringValue;
}

impl IntoOptionStringValue for Cow<'static, str> {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(Some(self))
	}
}

impl IntoOptionStringValue for String {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(Some(Cow::Owned(self)))
	}
}

impl IntoOptionStringValue for &'static str {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(Some(Cow::Borrowed(self)))
	}
}

impl IntoOptionStringValue for Option<Cow<'static, str>> {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(self)
	}
}

impl IntoOptionStringValue for Option<String> {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(self.map(Cow::Owned))
	}
}

impl IntoOptionStringValue for Option<&'static str> {
	fn into_option_string_value(self) -> OptionStringValue {
		OptionStringValue(self.map(Cow::Borrowed))
	}
}
