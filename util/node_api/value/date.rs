use crate::{Env, Value};

pub struct Date<'a>(Value<'a>);

impl<'a> Date<'a> {
	pub(crate) fn from_value(value: Value) -> Date {
		Date(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
