use crate::{Env, Value};

pub struct Symbol<'a>(Value<'a>);

impl<'a> Symbol<'a> {
	pub(crate) fn from_value(value: Value) -> Symbol {
		Symbol(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
