use crate::{
	sys::{
		napi_create_array, napi_get_array_length, napi_get_element, napi_set_element, napi_status,
	},
	Env, Error, Result, Value,
};
use num::ToPrimitive;
use std::mem::MaybeUninit;

#[derive(Clone, Copy)]
pub struct Array<'a>(Value<'a>);

impl<'a> Array<'a> {
	pub(crate) fn from_value(value: Value) -> Array {
		Array(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env) -> Result<Array> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_array(env.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		Ok(Array(Value::from_raw(env, value)))
	}

	pub fn size(&self) -> Result<usize> {
		let len = unsafe {
			let mut result = MaybeUninit::uninit();
			let status =
				napi_get_array_length(self.env().raw(), self.value().raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		let len = len
			.to_usize()
			.ok_or_else(|| Error::message("could not cast array length to usize"))?;
		Ok(len)
	}

	pub fn get(&self, index: usize) -> Result<Value<'a>> {
		let index = index
			.to_u32()
			.ok_or_else(|| Error::message("could not cast index to u32"))?;
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_element(
				self.env().raw(),
				self.value().raw(),
				index,
				result.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(self.env(), value);
		Ok(value)
	}

	pub fn set(&mut self, index: usize, value: impl Into<Value<'a>>) -> Result<()> {
		let index = index
			.to_u32()
			.ok_or_else(|| Error::message("could not cast index to u32"))?;
		let value = value.into();
		unsafe {
			let status = napi_set_element(self.env().raw(), self.value().raw(), index, value.raw());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
		}
		Ok(())
	}

	pub fn push(&mut self, value: impl Into<Value<'a>>) -> Result<()> {
		let len = self.size()?;
		self.set(len, value)?;
		Ok(())
	}

	pub fn iter(&self) -> Result<ArrayIterator<'a>> {
		Ok(ArrayIterator {
			array: *self,
			len: self.size()?,
			i: 0,
		})
	}
}

pub struct ArrayIterator<'a> {
	array: Array<'a>,
	len: usize,
	i: usize,
}

impl<'a> ArrayIterator<'a> {
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}
}

impl<'a> Iterator for ArrayIterator<'a> {
	type Item = Result<Value<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i >= self.len {
			None
		} else {
			let result = self.array.get(self.i);
			self.i += 1;
			Some(result)
		}
	}
}
