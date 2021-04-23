use crate::{Env, Value};

pub struct TypedArray<'a>(Value<'a>);

impl<'a> TypedArray<'a> {
	pub(crate) fn from_value(value: Value) -> TypedArray {
		TypedArray(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
