use node_api_sys::napi_status;

use crate::{
	sys::{napi_create_string_utf8, napi_get_value_string_utf8},
	Env, Error, Result, Value,
};
use std::{mem::MaybeUninit, os::raw::c_char};

#[derive(Clone, Copy)]
pub struct String<'a>(pub Value<'a>);

impl<'a> String<'a> {
	pub(crate) fn from_value(value: Value) -> String {
		String(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>, value: &str) -> Result<String<'a>> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_string_utf8(
				env.raw(),
				value.as_ptr() as *const c_char,
				value.len(),
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = String(value);
		Ok(value)
	}

	pub fn get(&self) -> Result<std::string::String> {
		let len = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_value_string_utf8(
				self.env().raw(),
				self.value().raw(),
				std::ptr::null_mut(),
				0,
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		let mut bytes: Vec<u8> = Vec::with_capacity(len + 1);
		let len = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_value_string_utf8(
				self.env().raw(),
				self.value().raw(),
				bytes.as_mut_ptr() as *mut c_char,
				bytes.capacity(),
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		unsafe { bytes.set_len(len) };
		let string = unsafe { std::string::String::from_utf8_unchecked(bytes) };
		Ok(string)
	}
}
