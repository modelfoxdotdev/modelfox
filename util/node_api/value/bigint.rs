use crate::{Env, Value};

pub struct BigInt<'a>(Value<'a>);

impl<'a> BigInt<'a> {
	pub(crate) fn from_value(value: Value) -> BigInt {
		BigInt(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
