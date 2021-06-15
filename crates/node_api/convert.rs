use crate::{
	Array, ArrayBuffer, BigInt, Boolean, Buffer, DataView, Date, Env, Error, External, Function,
	Null, Number, Object, Result, String, Symbol, TypedArray, Undefined, Value,
};
use num::{FromPrimitive, ToPrimitive};

#[allow(clippy::wrong_self_convention, clippy::upper_case_acronyms)]
pub trait ToNodeAPI<'a>: 'a {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>>;
}

#[allow(clippy::wrong_self_convention, clippy::upper_case_acronyms)]
pub trait FromNodeAPI<'a>: 'a + Sized {
	fn from_node_api(value: Value<'a>) -> Result<Self>;
}

impl<'a> ToNodeAPI<'a> for () {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		Ok(Null::new(env)?.value())
	}
}

impl<'a> FromNodeAPI<'a> for () {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_null()?;
		Ok(())
	}
}

impl<'a> ToNodeAPI<'a> for bool {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		Ok(Boolean::new(env, self)?.value())
	}
}

impl<'a> FromNodeAPI<'a> for bool {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		let value = value.as_boolean()?;
		let value = value.get()?;
		Ok(value)
	}
}

macro_rules! impl_to_from_for_number_type {
	($ty:ty) => {
		impl<'a> ToNodeAPI<'a> for $ty {
			fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
				let value =
					<$ty>::to_f64(&self).ok_or_else(|| Error::message("number out of bounds"))?;
				let number = Number::new(env, value)?;
				Ok(number.value())
			}
		}
		impl<'a> FromNodeAPI<'a> for $ty {
			fn from_node_api(value: Value<'a>) -> Result<Self> {
				let number = value.as_number()?;
				let value = number.get()?;
				let value =
					<$ty>::from_f64(value).ok_or_else(|| Error::message("number out of bounds"))?;
				Ok(value)
			}
		}
	};
}

impl_to_from_for_number_type!(usize);
impl_to_from_for_number_type!(u8);
impl_to_from_for_number_type!(u16);
impl_to_from_for_number_type!(u32);
impl_to_from_for_number_type!(u64);
impl_to_from_for_number_type!(isize);
impl_to_from_for_number_type!(i8);
impl_to_from_for_number_type!(i16);
impl_to_from_for_number_type!(i32);
impl_to_from_for_number_type!(i64);
impl_to_from_for_number_type!(f32);
impl_to_from_for_number_type!(f64);

impl<'a> ToNodeAPI<'a> for char {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		Ok(String::new(env, &self.to_string())?.value())
	}
}

impl<'a> FromNodeAPI<'a> for char {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		Ok(value.as_string()?.get()?.chars().next().unwrap())
	}
}

impl<'a> ToNodeAPI<'a> for &'a str {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		Ok(String::new(env, self)?.value())
	}
}

impl<'a> ToNodeAPI<'a> for std::string::String {
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		Ok(String::new(env, self.as_str())?.value())
	}
}

impl<'a> FromNodeAPI<'a> for std::string::String {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_string()?.get()
	}
}

impl<'a, T> ToNodeAPI<'a> for Option<T>
where
	T: ToNodeAPI<'a>,
{
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		match self {
			None => Ok(Null::new(env)?.value()),
			Some(value) => Ok(value.to_node_api(env)?),
		}
	}
}

impl<'a, T> FromNodeAPI<'a> for Option<T>
where
	T: FromNodeAPI<'a>,
{
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		if value.as_null().is_ok() || value.as_undefined().is_ok() {
			Ok(None)
		} else {
			Ok(Some(T::from_node_api(value)?))
		}
	}
}

impl<'a, T> ToNodeAPI<'a> for Vec<T>
where
	T: ToNodeAPI<'a>,
{
	fn to_node_api(self, env: Env<'a>) -> Result<Value<'a>> {
		let mut array = Array::new(env)?;
		for (i, value) in self.into_iter().enumerate() {
			array.set(i, value.to_node_api(env)?)?;
		}
		Ok(array.value())
	}
}

impl<'a, T> FromNodeAPI<'a> for Vec<T>
where
	T: FromNodeAPI<'a>,
{
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		let value = value.as_array()?;
		let mut vec = Vec::with_capacity(value.size()?);
		for value in value.iter()? {
			vec.push(T::from_node_api(value?)?);
		}
		Ok(vec)
	}
}

impl<'a> ToNodeAPI<'a> for Value<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self)
	}
}

impl<'a> FromNodeAPI<'a> for Value<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		Ok(value)
	}
}

impl<'a> ToNodeAPI<'a> for Array<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Array<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_array()
	}
}

impl<'a> ToNodeAPI<'a> for ArrayBuffer<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for ArrayBuffer<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_arraybuffer()
	}
}

impl<'a> ToNodeAPI<'a> for BigInt<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for BigInt<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_bigint()
	}
}

impl<'a> ToNodeAPI<'a> for Boolean<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Boolean<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_boolean()
	}
}

impl<'a> ToNodeAPI<'a> for Buffer<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Buffer<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_buffer()
	}
}

impl<'a> ToNodeAPI<'a> for DataView<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a, 'b: 'a> FromNodeAPI<'a> for DataView<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_dataview()
	}
}

impl<'a> ToNodeAPI<'a> for Date<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Date<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_date()
	}
}

impl<'a, T: 'a> ToNodeAPI<'a> for External<'a, T> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a, T: 'a> FromNodeAPI<'a> for External<'a, T> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_external()
	}
}

impl<'a> ToNodeAPI<'a> for Function<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Function<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_function()
	}
}

impl<'a> ToNodeAPI<'a> for Null<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Null<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_null()
	}
}

impl<'a> ToNodeAPI<'a> for Number<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Number<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_number()
	}
}

impl<'a> ToNodeAPI<'a> for Object<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Object<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_object()
	}
}

impl<'a> ToNodeAPI<'a> for String<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for String<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		let value = value.as_string()?;
		Ok(value)
	}
}

impl<'a> ToNodeAPI<'a> for Symbol<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Symbol<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_symbol()
	}
}

impl<'a> ToNodeAPI<'a> for TypedArray<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for TypedArray<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_typedarray()
	}
}

impl<'a> ToNodeAPI<'a> for Undefined<'a> {
	fn to_node_api(self, _env: Env<'a>) -> Result<Value<'a>> {
		Ok(self.value())
	}
}

impl<'a> FromNodeAPI<'a> for Undefined<'a> {
	fn from_node_api(value: Value<'a>) -> Result<Self> {
		value.as_undefined()
	}
}
