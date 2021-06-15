use std::mem::MaybeUninit;

use node_api_sys::napi_status;

use crate::{sys::napi_get_null, Env, Error, Result, Value};

pub struct Null<'a>(Value<'a>);

impl<'a> Null<'a> {
	pub(crate) fn from_value(value: Value) -> Null {
		Null(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>) -> Result<Null<'a>> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_null(env.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = Null(value);
		Ok(value)
	}
}
