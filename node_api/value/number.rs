use std::mem::MaybeUninit;

use node_api_sys::napi_status;

use crate::{
	sys::{napi_create_double, napi_get_value_double},
	Env, Error, Result, Value,
};

pub struct Number<'a>(Value<'a>);

impl<'a> Number<'a> {
	pub(crate) fn from_value(value: Value) -> Number {
		Number(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env, value: f64) -> Result<Number> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_double(env.raw(), value, result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = Number(value);
		Ok(value)
	}

	pub fn get(&self) -> Result<f64> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status =
				napi_get_value_double(self.env().raw(), self.value().raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		Ok(value)
	}
}
