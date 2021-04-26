use crate::{Env, Value};

pub struct DataView<'a>(Value<'a>);

impl<'a> DataView<'a> {
	pub(crate) fn from_value(value: Value) -> DataView {
		DataView(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
