use crate::{
	sys::{napi_create_arraybuffer, napi_get_arraybuffer_info, napi_status},
	Env, Error, Result, Value,
};
use std::mem::MaybeUninit;

pub struct ArrayBuffer<'a>(Value<'a>);

impl<'a> ArrayBuffer<'a> {
	pub(crate) fn from_value(value: Value) -> ArrayBuffer {
		ArrayBuffer(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>, bytes: &[u8]) -> Result<ArrayBuffer<'a>> {
		let len = bytes.len();
		let value = unsafe {
			let mut buffer = MaybeUninit::uninit();
			let mut value = MaybeUninit::uninit();
			let status =
				napi_create_arraybuffer(env.raw(), len, buffer.as_mut_ptr(), value.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			let buffer = buffer.assume_init();
			std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, len);
			value.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = ArrayBuffer(value);
		Ok(value)
	}

	pub fn get(&self) -> Result<&[u8]> {
		let (data, len) = unsafe {
			let mut data = MaybeUninit::uninit();
			let mut len = MaybeUninit::uninit();
			let status = napi_get_arraybuffer_info(
				self.env().raw(),
				self.value().raw(),
				data.as_mut_ptr(),
				len.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			(data.assume_init(), len.assume_init())
		};
		let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
		Ok(slice)
	}
}
