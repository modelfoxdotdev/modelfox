use crate::{
	sys::{napi_create_external, napi_env, napi_get_value_external, napi_status},
	Env, Error, Result, Value,
};
use std::{ffi::c_void, marker::PhantomData, mem::MaybeUninit};

pub struct External<'a, T>(Value<'a>, PhantomData<T>);

impl<'a, T> External<'a, T> {
	pub(crate) fn from_value(value: Value) -> External<T> {
		External(value, PhantomData)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env, value: T) -> Result<External<T>> {
		let value = Box::new(value);
		let value = Box::into_raw(value);
		unsafe extern "C" fn finalize<T>(_env: napi_env, data: *mut c_void, _hint: *mut c_void) {
			Box::from_raw(data as *mut T);
		}
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_external(
				env.raw(),
				value as *mut c_void,
				Some(finalize::<T>),
				std::ptr::null_mut(),
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let value = External(value, PhantomData);
		Ok(value)
	}

	pub fn get(&self) -> Result<&T> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status =
				napi_get_value_external(self.env().raw(), self.value().raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			};
			result.assume_init()
		};
		let value = value as *const T;
		let value = unsafe { &*value };
		Ok(value)
	}
}
