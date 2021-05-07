use crate::{Error, FromErlNif, List, ListIterator, Map, MapIterator, Result, Term};
use serde::{de::IntoDeserializer, Deserializer};

impl<'de> serde::de::Deserializer<'de> for Term<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self.is_atom() || self.is_binary() || self.is_bitstring() {
			self.deserialize_str(visitor)
		} else if self.is_float() {
			self.deserialize_f64(visitor)
		} else if self.is_fun() {
			Err(Error::message("cannot deserialize from fun"))
		} else if self.is_integer() {
			self.deserialize_i64(visitor)
		} else if self.is_list() {
			self.deserialize_seq(visitor)
		} else if self.is_map() {
			self.deserialize_map(visitor)
		} else if self.is_pid() {
			Err(Error::message("cannot deserialize from pid"))
		} else if self.is_port() {
			Err(Error::message("cannot deserialize from port"))
		} else if self.is_reference() {
			Err(Error::message("cannot deserialize from reference"))
		} else if self.is_tuple() {
			self.deserialize_seq(visitor)
		} else {
			Err(Error::message("unknown term type"))
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_bool(bool::from_erl_nif(self)?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i8(i8::from_erl_nif(self)?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i16(i16::from_erl_nif(self)?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i32(i32::from_erl_nif(self)?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_i64(i64::from_erl_nif(self)?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u8(u8::from_erl_nif(self)?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u16(u16::from_erl_nif(self)?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u32(u32::from_erl_nif(self)?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_u64(u64::from_erl_nif(self)?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f32(f32::from_erl_nif(self)?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_f64(f64::from_erl_nif(self)?)
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
		visitor.visit_string(String::from_erl_nif(self)?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_string(String::from_erl_nif(self)?)
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
		if self
			.as_atom()
			.and_then(|atom| atom.get())
			.map(|atom| atom == "nil")
			.unwrap_or(false)
		{
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if self
			.as_atom()
			.and_then(|atom| atom.get())
			.map(|atom| atom == "nil")
			.unwrap_or(false)
		{
			visitor.visit_unit()
		} else {
			Err(Error::message("expected :nil"))
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
		visitor.visit_seq(SeqDeserializer::new(self.as_list()?)?)
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
		visitor.visit_map(MapDeserializer::new(self.as_map()?)?)
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
		if self.is_tuple() {
			let tuple = self.as_tuple()?;
			let terms = tuple.get()?;
			let mut iter = terms.into_iter();
			let atom = iter
				.next()
				.ok_or_else(|| Error::message("enum must be a tuple with two values"))?;
			let atom = atom.as_atom()?;
			let variant = atom.get()?;
			let value = iter
				.next()
				.ok_or_else(|| Error::message("enum must be a tuple with two values"))?;
			visitor.visit_enum(EnumDeserializer {
				variant,
				value: Some(value),
			})
		} else if self.is_atom() {
			let variant = self.as_atom()?.get()?;
			visitor.visit_enum(EnumDeserializer {
				variant,
				value: None,
			})
		} else {
			Err(Error::message("expected tuple or atom"))
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
	iter: ListIterator<'de>,
}

impl<'de> SeqDeserializer<'de> {
	pub fn new(list: List<'de>) -> Result<SeqDeserializer<'de>> {
		let iter = list.iter();
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
			Some(value) => value,
			None => return Ok(None),
		};
		seed.deserialize(value).map(Some)
	}
}

struct MapDeserializer<'de> {
	len: usize,
	iter: MapIterator<'de>,
	value: Option<Term<'de>>,
}

impl<'de> MapDeserializer<'de> {
	pub fn new(map: Map<'de>) -> Result<MapDeserializer<'de>> {
		let len = map.size()?;
		let iter = map.iter()?;
		Ok(MapDeserializer {
			len,
			iter,
			value: None,
		})
	}
}

impl<'de> serde::de::MapAccess<'de> for MapDeserializer<'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: serde::de::DeserializeSeed<'de>,
	{
		let (key, value) = match self.iter.next() {
			Some((key, value)) => (key, value),
			None => return Ok(None),
		};
		self.value = Some(value);
		seed.deserialize(key).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		let value = self.value.take().unwrap();
		seed.deserialize(value)
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.len)
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
	value: Option<Term<'de>>,
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

impl<'de> serde::de::IntoDeserializer<'de, Error> for Term<'de> {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

struct VariantDeserializer<'de> {
	value: Option<Term<'de>>,
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
			Some(value) if value.is_list() => value.deserialize_seq(visitor),
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
			Some(value) if value.is_map() => value.deserialize_map(visitor),
			_ => Err(Error::message("expected struct variant")),
		}
	}
}
