use crate::{PointerType, Position, Read, StaticSize, Write, Writer};
use derive_deref::Deref;
use std::{convert::TryInto, marker::PhantomData, mem::size_of};

pub trait DynamicStructIndexType:
	num::Integer
	+ num::FromPrimitive
	+ num::ToPrimitive
	+ Into<PointerType>
	+ Clone
	+ Copy
	+ for<'a> Read<'a, Output = Self>
	+ Write<Output = Self>
{
}
impl DynamicStructIndexType for u8 {}
impl DynamicStructIndexType for u16 {}

#[derive(Clone, Copy, Debug)]
pub struct DynamicStructIndexReader<'a, I>
where
	I: DynamicStructIndexType,
{
	bytes: &'a [u8],
	position: Position<Self>,
	marker: PhantomData<I>,
}

impl<'a, I> DynamicStructIndexReader<'a, I>
where
	I: DynamicStructIndexType,
{
	pub fn new(bytes: &'a [u8], position: Position<Self>) -> DynamicStructIndexReader<'a, I> {
		DynamicStructIndexReader {
			bytes,
			position,
			marker: PhantomData,
		}
	}

	pub fn field_count(&self) -> DynamicStructIndexFieldCount<I> {
		let field_count_position = self.position.offset(0);
		DynamicStructIndexFieldCount::read(self.bytes, field_count_position)
	}

	pub fn get_field_offset(
		&self,
		field_id: DynamicStructIndexFieldId<I>,
	) -> DynamicStructIndexFieldOffset<I> {
		let offset = size_of::<DynamicStructIndexFieldCount<I>>()
			+ size_of::<DynamicStructIndexFieldOffset<I>>() * field_id.to_usize().unwrap();
		let offset = offset.try_into().unwrap();
		DynamicStructIndexFieldOffset::read(self.bytes, self.position.offset(offset))
	}
}

/// A `DynamicStructIndexPointer` points from a struct to its written vtable.
#[derive(Debug, Deref, Clone, Copy)]
pub struct DynamicStructIndexPointer(pub PointerType);

// A `DynamicStructIndexFieldCount` holds the number of fields in a vtable.
#[derive(Debug, Deref, Clone, Copy)]
pub struct DynamicStructIndexFieldCount<I>(pub I)
where
	I: DynamicStructIndexType;

/// A `FieldValueOffset` is the offset from the start of a struct to the value of a field.
#[derive(Debug, Deref, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DynamicStructIndexFieldOffset<I>(pub I)
where
	I: DynamicStructIndexType;

/// A `FieldId` is a unique identifier for each field.
#[derive(Debug, Deref, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicStructIndexFieldId<I>(pub I)
where
	I: DynamicStructIndexType;

impl<'a> StaticSize for DynamicStructIndexPointer {
	const STATIC_SIZE: PointerType = size_of::<PointerType>() as PointerType;
}

impl<'a> Read<'a> for DynamicStructIndexPointer {
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self {
		DynamicStructIndexPointer(PointerType::read(bytes, position.cast()))
	}
}

impl Write for DynamicStructIndexPointer {
	type Output = bool;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(&self.0).cast()
	}
}

impl<'a, I> StaticSize for DynamicStructIndexFieldCount<I>
where
	I: DynamicStructIndexType,
{
	const STATIC_SIZE: PointerType = 2;
}

impl<'a, I> Read<'a> for DynamicStructIndexFieldCount<I>
where
	I: DynamicStructIndexType,
{
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self {
		DynamicStructIndexFieldCount(I::read(bytes, position.cast()))
	}
}

impl<I> Write for DynamicStructIndexFieldCount<I>
where
	I: DynamicStructIndexType,
{
	type Output = Self;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(&self.0).cast()
	}
}

impl<'a, I> StaticSize for DynamicStructIndexFieldOffset<I>
where
	I: DynamicStructIndexType,
{
	const STATIC_SIZE: PointerType = 2;
}

impl<'a, I> Read<'a> for DynamicStructIndexFieldOffset<I>
where
	I: DynamicStructIndexType,
{
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		DynamicStructIndexFieldOffset(I::read(bytes, position.cast()))
	}
}

impl<I> Write for DynamicStructIndexFieldOffset<I>
where
	I: DynamicStructIndexType,
{
	type Output = Self;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(&self.0).cast()
	}
}
