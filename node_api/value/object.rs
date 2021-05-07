use crate::{
	sys::{
		napi_create_object, napi_get_property, napi_get_property_names, napi_set_property,
		napi_status,
	},
	Array, Env, Error, Result, Value,
};
use std::mem::MaybeUninit;

pub struct Object<'a>(pub(crate) Value<'a>);

impl<'a> Object<'a> {
	pub(crate) fn from_value(value: Value) -> Object {
		Object(value)
	}

	pub fn value(&self) -> Value<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env) -> Result<Object> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_create_object(env.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(env.raw(), status));
			}
			result.assume_init()
		};
		let value = Value::from_raw(env, value);
		let object = Object(value);
		Ok(object)
	}

	pub fn properties(&self) -> Result<Array<'a>> {
		let properties = unsafe {
			let mut properties = MaybeUninit::uninit();
			let status = napi_get_property_names(
				self.env().raw(),
				self.value().raw(),
				properties.as_mut_ptr(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			properties.assume_init()
		};
		let properties = Array::from_value(Value::from_raw(self.env(), properties));
		Ok(properties)
	}

	pub fn size(&self) -> Result<usize> {
		let properties = self.properties()?;
		let len = properties.size()?;
		Ok(len)
	}

	pub fn get(&self, key: impl Into<Value<'a>>) -> Result<Value<'a>> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_get_property(
				self.env().raw(),
				self.value().raw(),
				key.into().raw(),
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

	pub fn set(&mut self, key: impl Into<Value<'a>>, value: impl Into<Value<'a>>) -> Result<()> {
		unsafe {
			let status = napi_set_property(
				self.env().raw(),
				self.value().raw(),
				key.into().raw(),
				value.into().raw(),
			);
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
		};
		Ok(())
	}
}
