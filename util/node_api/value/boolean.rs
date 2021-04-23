use std::mem::MaybeUninit;

use node_api_sys::napi_status;

use crate::{
	sys::{napi_get_boolean, napi_get_value_bool},
	Env, Error, Result, Value,
};

pub struct Boolean<'a>(Value<'a>);

impl<'a> Boolean<'a> {
	pub(crate) fn from_value(value: Value) -> Boolean {
		Boolean(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env, value: bool) -> Result<Boolean> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_boolean(env.raw(), value, result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = Boolean(value);
		Ok(value)
	}

	pub fn get(&self) -> Result<bool> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status =
				napi_get_value_bool(self.env().raw(), self.value().raw(), result.as_mut_ptr());

			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		Ok(value)
	}
}
