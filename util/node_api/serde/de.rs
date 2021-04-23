use crate::{Array, ArrayIterator, Error, FromNodeAPI, Object, Result, Value};
use serde::{de::IntoDeserializer, Deserializer};

impl<'de> serde::de::Deserializer<'de> for Value<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self.is_array()? {
			self.deserialize_seq(visitor)
		} else if self.is_arraybuffer()? {
			Err(Error::message("cannot deserialize from arraybuffer"))
		} else if self.is_bigint()? {
			Err(Error::message("cannot deserialize from bigint"))
		} else if self.is_boolean()? {
			self.deserialize_bool(visitor)
		} else if self.is_buffer()? {
			Err(Error::message("cannot deserialize from buffer"))
		} else if self.is_dataview()? {
			Err(Error::message("cannot deserialize from dataview"))
		} else if self.is_date()? {
			Err(Error::message("cannot deserialize from date"))
		} else if self.is_external()? {
			Err(Error::message("cannot deserialize from external"))
		} else if self.is_function()? {
			Err(Error::message("cannot deserialize from function"))
		} else if self.is_null()? {
			self.deserialize_unit(visitor)
		} else if self.is_number()? {
			self.deserialize_f64(visitor)
		} else if self.is_object()? {
			self.deserialize_map(visitor)
		} else if self.is_string()? {
			self.deserialize_string(visitor)
		} else if self.is_symbol()? {
			Err(Error::message("cannot deserialize from symbol"))
		} else if self.is_typedarray()? {
			Err(Error::message("cannot deserialize from typedarray"))
		} else if self.is_undefined()? {
			self.deserialize_unit(visitor)
		} else {
			Err(Error::message("unknown value type"))
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bool(bool::from_node_api(self)?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i8(i8::from_node_api(self)?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i16(i16::from_node_api(self)?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i32(i32::from_node_api(self)?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i64(i64::from_node_api(self)?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u8(u8::from_node_api(self)?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u16(u16::from_node_api(self)?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u32(u32::from_node_api(self)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u64(u64::from_node_api(self)?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f32(f32::from_node_api(self)?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f64(f64::from_node_api(self)?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_string(String::from_node_api(self)?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_string(String::from_node_api(self)?)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self.is_null()? || self.is_undefined()? {
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self.is_null()? || self.is_undefined()? {
			visitor.visit_unit()
		} else {
			Err(Error::message("expected null or undefined"))
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_seq(SeqDeserializer::new(self.as_array()?)?)
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_map(MapDeserializer::new(self.as_object()?)?)
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self.is_object()? {
			let object = self.as_object()?;
			let properties = object.properties()?;
			let property = match properties.iter()?.next() {
				Some(Ok(property)) => property,
				Some(Err(error)) => return Err(error),
				None => return Err(Error::message("enum must be an object with one property")),
			};
			let value = object.get(property)?;
			visitor.visit_enum(EnumDeserializer {
				variant: property.as_string()?.get()?,
				value: Some(value),
			})
		} else if self.is_string()? {
			let variant = self.as_string()?.get()?;
			visitor.visit_enum(EnumDeserializer {
				variant,
				value: None,
			})
		} else {
			Err(Error::message("expected object or string"))
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}
}

struct SeqDeserializer<'de> {
	iter: ArrayIterator<'de>,
}

impl<'de> SeqDeserializer<'de> {
	pub fn new(array: Array<'de>) -> Result<SeqDeserializer<'de>> {
		let iter = array.iter()?;
		Ok(SeqDeserializer { iter })
	}
}

impl<'de> serde::de::SeqAccess<'de> for SeqDeserializer<'de> {
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		let value = match self.iter.next() {
			Some(Ok(value)) => value,
			Some(Err(error)) => return Err(error),
			None => return Ok(None),
		};
		seed.deserialize(value).map(Some)
	}
}

struct MapDeserializer<'de> {
	object: Object<'de>,
	properties: ArrayIterator<'de>,
	key: Option<Value<'de>>,
}

impl<'de> MapDeserializer<'de> {
	pub fn new(object: Object<'de>) -> Result<MapDeserializer<'de>> {
		let properties = object.properties()?.iter()?;
		Ok(MapDeserializer {
			object,
			properties,
			key: None,
		})
	}
}

impl<'de> serde::de::MapAccess<'de> for MapDeserializer<'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: serde::de::DeserializeSeed<'de>,
	{
		let key = match self.properties.next() {
			Some(Ok(key)) => key,
			Some(Err(error)) => return Err(error),
			None => return Ok(None),
		};
		self.key = Some(key);
		seed.deserialize(key).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		let key = self.key.take().unwrap();
		let value = self.object.get(key)?;
		seed.deserialize(value)
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.properties.len())
	}
}

impl serde::de::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: std::fmt::Display,
	{
		Error::message(msg.to_string())
	}
}

struct EnumDeserializer<'de> {
	variant: String,
	value: Option<Value<'de>>,
}

impl<'de> serde::de::EnumAccess<'de> for EnumDeserializer<'de> {
	type Error = Error;
	type Variant = VariantDeserializer<'de>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer<'de>), Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		let visitor = VariantDeserializer { value: self.value };
		seed.deserialize(self.variant.into_deserializer())
			.map(|v| (v, visitor))
	}
}

impl<'de> serde::de::IntoDeserializer<'de, Error> for Value<'de> {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

struct VariantDeserializer<'de> {
	value: Option<Value<'de>>,
}

impl<'de> serde::de::VariantAccess<'de> for VariantDeserializer<'de> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Error> {
		match self.value {
			Some(value) => serde::Deserialize::deserialize(value),
			None => Ok(()),
		}
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		match self.value {
			Some(value) => seed.deserialize(value),
			None => Err(Error::message("expected newtype variant")),
		}
	}

	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.value {
			Some(value) if value.is_array()? => value.deserialize_seq(visitor),
			_ => Err(Error::message("expected tuple variant")),
		}
	}

	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.value {
			Some(value) if value.is_object()? => value.deserialize_map(visitor),
			_ => Err(Error::message("expected struct variant")),
		}
	}
}
