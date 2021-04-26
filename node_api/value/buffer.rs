use crate::{Env, Value};

pub struct Buffer<'a>(Value<'a>);

impl<'a> Buffer<'a> {
	pub(crate) fn from_value(value: Value) -> Buffer {
		Buffer(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
