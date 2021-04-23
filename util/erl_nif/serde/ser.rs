use crate::{Atom, BinaryTerm, Env, Error, List, Map, Result, Term, ToErlNif, Tuple};

impl<'a> serde::Serializer for Env<'a> {
	type Ok = Term<'a>;
	type Error = Error;
	type SerializeSeq = SeqSerializer<'a>;
	type SerializeTuple = TupleSerializer<'a>;
	type SerializeTupleStruct = TupleStructSerializer<'a>;
	type SerializeTupleVariant = TupleVariantSerializer<'a>;
	type SerializeMap = MapSerializer<'a>;
	type SerializeStruct = StructSerializer<'a>;
	type SerializeStructVariant = StructVariantSerializer<'a>;

	fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
		value.to_erl_nif(self)
	}

	fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
		Ok(BinaryTerm::from_str(self, value)?.term())
	}

	fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(BinaryTerm::from_bytes(self, value)?.term())
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(Atom::new(self, "nil")?.term())
	}

	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: serde::Serialize,
	{
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Ok(Atom::new(self, "nil")?.term())
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
		let atom = Atom::new(self, variant)?;
		let tuple = Tuple::new(self, vec![atom.term()])?;
		Ok(tuple.term())
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
		let atom = Atom::new(self, variant)?;
		let value = value.serialize(self)?;
		let tuple = Tuple::new(self, vec![atom.term(), value])?;
		Ok(tuple.term())
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
		name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		StructSerializer::new(self, name, len)
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
	env: Env<'a>,
	terms: Vec<Term<'a>>,
}

impl<'a> SeqSerializer<'a> {
	pub fn new(env: Env, _len: Option<usize>) -> Result<SeqSerializer> {
		Ok(SeqSerializer {
			env,
			terms: Vec::new(),
		})
	}
}

impl<'a> serde::ser::SerializeSeq for SeqSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.env)?;
		self.terms.push(value);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(List::new(self.env, self.terms)?.term())
	}
}

pub struct TupleSerializer<'a> {
	env: Env<'a>,
	terms: Vec<Term<'a>>,
}

impl<'a> TupleSerializer<'a> {
	pub fn new(env: Env, _len: usize) -> Result<TupleSerializer> {
		Ok(TupleSerializer {
			env,
			terms: Vec::new(),
		})
	}
}

impl<'a> serde::ser::SerializeTuple for TupleSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.env)?;
		self.terms.push(value);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Tuple::new(self.env, self.terms)?.term())
	}
}

pub struct TupleStructSerializer<'a> {
	env: Env<'a>,
	terms: Vec<Term<'a>>,
}

impl<'a> TupleStructSerializer<'a> {
	pub fn new(env: Env, _len: usize) -> Result<TupleStructSerializer> {
		Ok(TupleStructSerializer {
			env,
			terms: Vec::new(),
		})
	}
}

impl<'a> serde::ser::SerializeTupleStruct for TupleStructSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.env)?;
		self.terms.push(value);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Tuple::new(self.env, self.terms)?.term())
	}
}

pub struct TupleVariantSerializer<'a> {
	env: Env<'a>,
	variant: Atom<'a>,
	terms: Vec<Term<'a>>,
}

impl<'a> TupleVariantSerializer<'a> {
	pub fn new(env: Env<'a>, variant: &str, _len: usize) -> Result<TupleVariantSerializer<'a>> {
		let variant = Atom::new(env, variant)?;
		Ok(TupleVariantSerializer {
			env,
			variant,
			terms: Vec::new(),
		})
	}
}

impl<'a> serde::ser::SerializeTupleVariant for TupleVariantSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let value = value.serialize(self.env)?;
		self.terms.push(value);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let tuple = Tuple::new(self.env, self.terms)?;
		let tuple = Tuple::new(self.env, vec![self.variant.term(), tuple.term()])?;
		Ok(tuple.term())
	}
}

pub struct MapSerializer<'a> {
	map: Map<'a>,
	key: Option<Term<'a>>,
}

impl<'a> MapSerializer<'a> {
	pub fn new(env: Env, _len: Option<usize>) -> Result<MapSerializer> {
		let map = Map::new(env)?;
		Ok(MapSerializer { map, key: None })
	}
}

impl<'a> serde::ser::SerializeMap for MapSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		self.key = Some(key.serialize(self.map.env())?);
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
		let value = value.serialize(self.map.env())?;
		self.map.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.map.term())
	}
}

pub struct StructSerializer<'a> {
	map: Map<'a>,
}

impl<'a> StructSerializer<'a> {
	pub fn new(env: Env<'a>, name: &str, _len: usize) -> Result<StructSerializer<'a>> {
		let mut map = Map::new(env)?;
		let struct_name_key = Atom::new(env, "__struct__")?;
		let struct_name_value = Atom::new(env, name)?;
		map.set(struct_name_key, struct_name_value)?;
		Ok(StructSerializer { map })
	}
}

impl<'a> serde::ser::SerializeStruct for StructSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let key = Atom::new(self.map.env(), key)?;
		let value = value.serialize(self.map.env())?;
		self.map.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.map.term())
	}
}

pub struct StructVariantSerializer<'a> {
	map: Map<'a>,
}

impl<'a> StructVariantSerializer<'a> {
	pub fn new(env: Env, _len: usize) -> Result<StructVariantSerializer> {
		let map = Map::new(env)?;
		Ok(StructVariantSerializer { map })
	}
}

impl<'a> serde::ser::SerializeStructVariant for StructVariantSerializer<'a> {
	type Ok = Term<'a>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: serde::Serialize,
	{
		let key = serde::Serializer::serialize_str(self.map.env(), key)?;
		let value = value.serialize(self.map.env())?;
		self.map.set(key, value)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.map.term())
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
