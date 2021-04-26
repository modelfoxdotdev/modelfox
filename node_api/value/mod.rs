use crate::{
	sys::{
		napi_is_array, napi_is_arraybuffer, napi_is_buffer, napi_is_dataview, napi_is_date,
		napi_is_typedarray, napi_status, napi_typeof, napi_value, napi_valuetype,
	},
	Env, Error, Result,
};
use std::mem::MaybeUninit;

pub mod array;
pub mod arraybuffer;
pub mod bigint;
pub mod boolean;
pub mod buffer;
pub mod dataview;
pub mod date;
pub mod external;
pub mod function;
pub mod null;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;
pub mod typedarray;
pub mod undefined;

pub use self::{
	array::Array, arraybuffer::ArrayBuffer, bigint::BigInt, boolean::Boolean, buffer::Buffer,
	dataview::DataView, date::Date, external::External, function::Function, null::Null,
	number::Number, object::Object, string::String, symbol::Symbol, typedarray::TypedArray,
	undefined::Undefined,
};

#[derive(Clone, Copy)]
pub struct Value<'a> {
	env: Env<'a>,
	value: napi_value,
}

impl<'a> Value<'a> {
	pub fn from_raw(env: Env<'a>, value: napi_value) -> Value<'a> {
		Value { env, value }
	}

	pub fn env(&self) -> Env<'a> {
		self.env
	}

	pub fn raw(&self) -> napi_value {
		self.value
	}

	fn value_type(&self) -> Result<napi_valuetype> {
		let value = unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_typeof(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			result.assume_init()
		};
		Ok(value)
	}

	pub fn is_array(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_array(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_array(&self) -> Result<Array<'a>> {
		if self.is_array()? {
			Ok(Array::from_value(*self))
		} else {
			Err(Error::message("could not cast value to array"))
		}
	}

	pub fn is_arraybuffer(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_arraybuffer(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_arraybuffer(&self) -> Result<ArrayBuffer<'a>> {
		if self.is_arraybuffer()? {
			Ok(ArrayBuffer::from_value(*self))
		} else {
			Err(Error::message("could not cast value to arraybuffer"))
		}
	}

	pub fn is_bigint(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_bigint)
	}

	pub fn as_bigint(&self) -> Result<BigInt<'a>> {
		if self.is_bigint()? {
			Ok(BigInt::from_value(*self))
		} else {
			Err(Error::message("could not cast value to bigint"))
		}
	}

	pub fn is_boolean(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_boolean)
	}

	pub fn as_boolean(&self) -> Result<Boolean<'a>> {
		if self.is_boolean()? {
			Ok(Boolean::from_value(*self))
		} else {
			Err(Error::message("could not cast value to boolean"))
		}
	}

	pub fn is_buffer(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_buffer(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_buffer(&self) -> Result<Buffer<'a>> {
		if self.is_buffer()? {
			Ok(Buffer::from_value(*self))
		} else {
			Err(Error::message("could not cast value to buffer"))
		}
	}

	pub fn is_dataview(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_dataview(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_dataview(&self) -> Result<DataView<'a>> {
		if self.is_dataview()? {
			Ok(DataView::from_value(*self))
		} else {
			Err(Error::message("could not cast value to dataview"))
		}
	}

	pub fn is_date(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_date(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_date(&self) -> Result<Date<'a>> {
		if self.is_date()? {
			Ok(Date::from_value(*self))
		} else {
			Err(Error::message("could not cast value to date"))
		}
	}

	pub fn is_external(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_external)
	}

	pub fn as_external<T>(&self) -> Result<External<'a, T>> {
		if self.is_external()? {
			Ok(External::from_value(*self))
		} else {
			Err(Error::message("could not cast value to external"))
		}
	}

	pub fn is_function(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_function)
	}

	pub fn as_function(&self) -> Result<Function<'a>> {
		if self.is_function()? {
			Ok(Function::from_value(*self))
		} else {
			Err(Error::message("could not cast value to function"))
		}
	}

	pub fn is_null(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_null)
	}

	pub fn as_null(&self) -> Result<Null<'a>> {
		if self.is_null()? {
			Ok(Null::from_value(*self))
		} else {
			Err(Error::message("could not cast value to null"))
		}
	}

	pub fn is_number(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_number)
	}

	pub fn as_number(&self) -> Result<Number<'a>> {
		if self.is_number()? {
			Ok(Number::from_value(*self))
		} else {
			Err(Error::message("could not cast value to number"))
		}
	}

	pub fn is_object(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_object)
	}

	pub fn as_object(&self) -> Result<Object<'a>> {
		if self.is_object()? {
			Ok(Object::from_value(*self))
		} else {
			Err(Error::message("could not cast value to object"))
		}
	}

	pub fn is_string(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_string)
	}

	pub fn as_string(&self) -> Result<String<'a>> {
		if self.is_string()? {
			Ok(String::from_value(*self))
		} else {
			Err(Error::message("could not cast value to string"))
		}
	}

	pub fn is_symbol(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_symbol)
	}

	pub fn as_symbol(&self) -> Result<Symbol<'a>> {
		if self.is_symbol()? {
			Ok(Symbol::from_value(*self))
		} else {
			Err(Error::message("could not cast value to symbol"))
		}
	}

	pub fn is_typedarray(&self) -> Result<bool> {
		unsafe {
			let mut result = MaybeUninit::uninit();
			let status = napi_is_typedarray(self.env().raw(), self.raw(), result.as_mut_ptr());
			if status != napi_status::napi_ok {
				return Err(Error::from_last_node_api_error(self.env().raw(), status));
			}
			Ok(result.assume_init())
		}
	}

	pub fn as_typedarray(&self) -> Result<TypedArray<'a>> {
		if self.is_typedarray()? {
			Ok(TypedArray::from_value(*self))
		} else {
			Err(Error::message("could not cast value to typedarray"))
		}
	}

	pub fn is_undefined(&self) -> Result<bool> {
		Ok(self.value_type()? == napi_valuetype::napi_undefined)
	}

	pub fn as_undefined(&self) -> Result<Undefined<'a>> {
		if self.is_undefined()? {
			Ok(Undefined::from_value(*self))
		} else {
			Err(Error::message("could not cast value to undefined"))
		}
	}
}

impl<'a> From<Array<'a>> for Value<'a> {
	fn from(value: Array<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<ArrayBuffer<'a>> for Value<'a> {
	fn from(value: ArrayBuffer<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<BigInt<'a>> for Value<'a> {
	fn from(value: BigInt<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Boolean<'a>> for Value<'a> {
	fn from(value: Boolean<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Buffer<'a>> for Value<'a> {
	fn from(value: Buffer<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<DataView<'a>> for Value<'a> {
	fn from(value: DataView<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Date<'a>> for Value<'a> {
	fn from(value: Date<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a, T> From<External<'a, T>> for Value<'a>
where
	T: 'a,
{
	fn from(value: External<'a, T>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Function<'a>> for Value<'a> {
	fn from(value: Function<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Null<'a>> for Value<'a> {
	fn from(value: Null<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Number<'a>> for Value<'a> {
	fn from(value: Number<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Object<'a>> for Value<'a> {
	fn from(value: Object<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<String<'a>> for Value<'a> {
	fn from(value: String<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Symbol<'a>> for Value<'a> {
	fn from(value: Symbol<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<TypedArray<'a>> for Value<'a> {
	fn from(value: TypedArray<'a>) -> Value<'a> {
		value.value()
	}
}

impl<'a> From<Undefined<'a>> for Value<'a> {
	fn from(value: Undefined<'a>) -> Value<'a> {
		value.value()
	}
}
