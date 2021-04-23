use crate::{Array, ArrayBuffer, Env, Error, Null, Object, Result, String, ToNodeAPI, Value};
use serde::Serialize;

impl<'a> serde::Serializer for Env<'a> {
	type Ok = Value<'a>;
	type Error = Error;
	type SerializeSeq = SeqSerializer<'a>;
	type SerializeTuple = TupleSerializer<'a>;
	type SerializeTupleStruct = TupleStructSerializer<'a>;
	type SerializeTupleVariant = TupleVariantSerializer<'a>;
	type SerializeMap = MapSerializer<'a>;
	type SerializeStruct = StructSerializer<'a>;
	type SerializeStructVariant = StructVariantSerializer<'a>;

	fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
		value.to_node_api(self)
	}

	fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
		Ok(String::new(self, value)?.value())
	}

	fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(ArrayBuffer::new(self, value)?.value())
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(Null::new(self)?.value())
	}

	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: serde::Serialize,
	{
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Ok(Null::new(self)?.value())
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}

	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		self.serialize_str(variant)
	}

	fn serialize_newtype_struct<T: ?Sized>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: serde::Serialize,
	{
		value.serialize(self)
	}

	fn serialize_newtype_variant<T: ?Sized>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: serde::Serialize,
	{
		let mut object = Object::new(self)?;
		let key = variant.serialize(self)?;
		let value = value.serialize(self)?;
		object.set(key, value)?;
		Ok(object.value())
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		SeqSerializer::new(self, len)
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		TupleSerializer::new(self, len)
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		TupleStructSerializer::new(self, len)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		TupleVariantSerializer::new(self, variant, len)
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		MapSerializer::new(self, len)
	}

	fn serialize_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		StructSerializer::new(self, len)
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		StructVariantSerializer::new(self, len)
	}
}

pub struct SeqSerializer<'a> {
	array: Array<'a>,
}

impl<'a> SeqSerializer<'a> {
	pub fn new(env: Env, _len: Option<usize>) -> Result<SeqSerializer> {
		let array = Array::new(env)?;
		Ok(SeqSerializer { array })
	}
}

impl<'a> serde::ser::SerializeSeq for SeqSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.array.env())?;
		self.array.push(value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.array.value())
	}
}

pub struct TupleSerializer<'a> {
	array: Array<'a>,
}

impl<'a> TupleSerializer<'a> {
	pub fn new(env: Env, _len: usize) -> Result<TupleSerializer> {
		let array = Array::new(env)?;
		Ok(TupleSerializer { array })
	}
}

impl<'a> serde::ser::SerializeTuple for TupleSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.array.env())?;
		self.array.push(value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.array.value())
	}
}

pub struct TupleStructSerializer<'a> {
	array: Array<'a>,
}

impl<'a> TupleStructSerializer<'a> {
	pub fn new(env: Env, _len: usize) -> Result<TupleStructSerializer> {
		let array = Array::new(env)?;
		Ok(TupleStructSerializer { array })
	}
}

impl<'a> serde::ser::SerializeTupleStruct for TupleStructSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.array.env())?;
		self.array.push(value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.array.value())
	}
}

pub struct TupleVariantSerializer<'a> {
	env: Env<'a>,
	variant: String<'a>,
	array: Array<'a>,
}

impl<'a> TupleVariantSerializer<'a> {
	pub fn new(env: Env<'a>, variant: &str, _len: usize) -> Result<TupleVariantSerializer<'a>> {
		let variant = String::new(env, variant)?;
		let array = Array::new(env)?;
		Ok(TupleVariantSerializer {
			env,
			variant,
			array,
		})
	}
}

impl<'a> serde::ser::SerializeTupleVariant for TupleVariantSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.array.env())?;
		self.array.push(value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let mut object = Object::new(self.env)?;
		object.set(self.variant, self.array)?;
		Ok(object.value())
	}
}

pub struct MapSerializer<'a> {
	object: Object<'a>,
	key: Option<Value<'a>>,
}

impl<'a> MapSerializer<'a> {
	pub fn new(env: Env<'a>, _len: Option<usize>) -> Result<MapSerializer<'a>> {
		let object = Object::new(env)?;
		Ok(MapSerializer { object, key: None })
	}
}

impl<'a> serde::ser::SerializeMap for MapSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		self.key = Some(key.serialize(self.object.env())?);
		Ok(())
	}

	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let key = self
			.key
			.take()
			.expect("serialize_value called before serialize_key");
		let value = value.serialize(self.object.env())?;
		self.object.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.object.value())
	}
}

pub struct StructSerializer<'a> {
	object: Object<'a>,
}

impl<'a> StructSerializer<'a> {
	pub fn new(env: Env<'a>, _len: usize) -> Result<StructSerializer<'a>> {
		let object = Object::new(env)?;
		Ok(StructSerializer { object })
	}
}

impl<'a> serde::ser::SerializeStruct for StructSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let key = serde::Serializer::serialize_str(self.object.env(), key)?;
		let value = value.serialize(self.object.env())?;
		self.object.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.object.value())
	}
}

pub struct StructVariantSerializer<'a> {
	object: Object<'a>,
}

impl<'a> StructVariantSerializer<'a> {
	pub fn new(env: Env<'a>, _len: usize) -> Result<StructVariantSerializer<'a>> {
		let object = Object::new(env)?;
		Ok(StructVariantSerializer { object })
	}
}

impl<'a> serde::ser::SerializeStructVariant for StructVariantSerializer<'a> {
	type Ok = Value<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let key = serde::Serializer::serialize_str(self.object.env(), key)?;
		let value = value.serialize(self.object.env())?;
		self.object.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.object.value())
	}
}

impl serde::ser::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: std::fmt::Display,
	{
		Error::message(msg.to_string())
	}
}
