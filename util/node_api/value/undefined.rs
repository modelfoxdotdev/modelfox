use std::mem::MaybeUninit;

use node_api_sys::napi_status;

use crate::{sys::napi_get_undefined, Env, Error, Result, Value};

pub struct Undefined<'a>(Value<'a>);

impl<'a> Undefined<'a> {
	pub(crate) fn from_value(value: Value) -> Undefined {
		Undefined(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env) -> Result<Undefined> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_undefined(env.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = Undefined(value);
		Ok(value)
	}
}
