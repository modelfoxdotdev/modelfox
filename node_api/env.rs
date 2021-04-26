use crate::{
	fatal_error,
	sys::{napi_env, napi_get_global, napi_is_exception_pending, napi_status, napi_throw_error},
	Error, Result, Value,
};
use std::{marker::PhantomData, mem::MaybeUninit};

#[derive(Clone, Copy)]
pub struct Env<'a>(napi_env, PhantomData<&'a napi_env>);

impl<'a> Env<'a> {
	pub fn from_raw(env: napi_env) -> Env<'a> {
		Env(env, PhantomData)
	}

	pub fn raw(&self) -> napi_env {
		self.0
	}

	pub fn global(&self) -> Result<Value> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_global(self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(*self, value);
		Ok(value)
	}

	pub fn is_exception_pending(&self) -> bool {
		unsafe {
			let mut is_exception_pending = std::mem::MaybeUninit::uninit();
			let status = napi_is_exception_pending(self.raw(), is_exception_pending.as_mut_ptr());
			if status != napi_status::napi_ok {
				fatal_error(None, None);
				unreachable!()
			}
			is_exception_pending.assume_init()
		}
	}

	pub fn throw_error(&self, message: &str) {
		unsafe {
			let message = std::ffi::CString::new(message);
			let message = match message {
				Ok(message) => message,
				Err(_) => {
					fatal_error(None, None);
					unreachable!()
				}
			};
			let status = napi_throw_error(self.raw(), std::ptr::null(), message.as_ptr());
			if status != napi_status::napi_ok {
				fatal_error(None, None);
				unreachable!()
			}
		}
	}
}
