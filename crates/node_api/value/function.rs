use node_api_sys::napi_status;

use crate::{
	sys::{napi_callback_info, napi_create_function, napi_env, napi_value},
	Env, Error, Result, Value,
};
use std::{mem::MaybeUninit, os::raw::c_char};

pub struct Function<'a>(Value<'a>);

impl<'a> Function<'a> {
	pub(crate) fn from_value(value: Value) -> Function {
		Function(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(
		env: Env<'a>,
		name: &str,
		value: unsafe extern "C" fn(env: napi_env, info: napi_callback_info) -> napi_value,
	) -> Result<Function<'a>> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_function(
				env.raw(),
				name.as_ptr() as *const c_char,
				name.len(),
				Some(value),
				std::ptr::null_mut(),
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = Function(value);
		Ok(value)
	}
}
