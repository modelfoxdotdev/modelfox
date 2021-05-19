use std::borrow::Cow;

#[derive(Clone)]
pub enum AttributeValue {
	Bool(Option<bool>),
	String(Option<Cow<'static, str>>),
}

pub trait IntoAttributeValue {
	fn into_attribute_value(self) -> AttributeValue;
}

impl IntoAttributeValue for bool {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::Bool(Some(self))
	}
}

impl IntoAttributeValue for Option<bool> {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::Bool(self)
	}
}

impl IntoAttributeValue for Cow<'static, str> {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(Some(self))
	}
}

impl IntoAttributeValue for String {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(Some(Cow::Owned(self)))
	}
}

impl IntoAttributeValue for &'static str {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(Some(Cow::Borrowed(self)))
	}
}

impl IntoAttributeValue for Option<Cow<'static, str>> {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(self)
	}
}

impl IntoAttributeValue for Option<String> {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(self.map(Cow::Owned))
	}
}

impl IntoAttributeValue for Option<&'static str> {
	fn into_attribute_value(self) -> AttributeValue {
		AttributeValue::String(self.map(Cow::Borrowed))
	}
}
