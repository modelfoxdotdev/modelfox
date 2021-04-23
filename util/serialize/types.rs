use crate::{
	Pointer, PointerType, Position, Read, ReadType, StaticSize, VecReader, Write, WriteType, Writer,
};
use num::ToPrimitive;
use std::{convert::TryInto, mem::size_of};

impl<'a> ReadType<'a> for () {
	type ReadType = ();
}

impl WriteType for () {
	type WriteType = ();
}

impl StaticSize for () {
	const STATIC_SIZE: PointerType = 0;
}

impl<'a> Read<'a> for () {
	type Output = Self;
	fn read(_bytes: &'a [u8], _position: Position<Self>) -> Self::Output {}
}

impl Write for () {
	type Output = Self;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.position()
	}
}

impl<'a> ReadType<'a> for bool {
	type ReadType = bool;
}

impl WriteType for bool {
	type WriteType = bool;
}

impl StaticSize for bool {
	const STATIC_SIZE: PointerType = 1;
}

impl<'a> Read<'a> for bool {
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		let position = position.cast();
		<u8>::read(bytes, position) != 0
	}
}

impl Write for bool {
	type Output = Self;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let value = *self as u8;
		writer.write_raw::<bool>(&value.to_le_bytes())
	}
}

macro_rules! impl_read_write_for_number_type {
	($ty:ty) => {
		impl<'a> ReadType<'a> for $ty {
			type ReadType = $ty;
		}
		impl WriteType for $ty {
			type WriteType = $ty;
		}
		impl StaticSize for $ty {
			const STATIC_SIZE: PointerType = size_of::<$ty>() as PointerType;
		}
		impl<'a> Read<'a> for $ty {
			type Output = Self;
			fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
				let position = position.to_usize().unwrap();
				let bytes = &bytes[position..position + size_of::<$ty>()];
				let bytes = bytes.try_into().unwrap();
				<$ty>::from_le_bytes(bytes)
			}
		}
		impl Write for $ty {
			type Output = $ty;
			fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
				writer.write_raw::<$ty>(&self.to_le_bytes())
			}
		}
	};
}

impl_read_write_for_number_type!(u8);
impl_read_write_for_number_type!(u16);
impl_read_write_for_number_type!(u32);
impl_read_write_for_number_type!(u64);
impl_read_write_for_number_type!(i8);
impl_read_write_for_number_type!(i16);
impl_read_write_for_number_type!(i32);
impl_read_write_for_number_type!(i64);
impl_read_write_for_number_type!(f32);
impl_read_write_for_number_type!(f64);

impl<'a> ReadType<'a> for char {
	type ReadType = char;
}

impl WriteType for char {
	type WriteType = char;
}

impl StaticSize for char {
	const STATIC_SIZE: PointerType = 4;
}

impl<'a> Read<'a> for char {
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		<u32>::read(bytes, position.cast()).try_into().unwrap()
	}
}

impl Write for char {
	type Output = Self;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(&(*self as u32)).cast()
	}
}

impl<'a, T> ReadType<'a> for Option<T>
where
	T: ReadType<'a>,
	T::ReadType: StaticSize,
{
	type ReadType = Option<T::ReadType>;
}

impl<T> WriteType for Option<T>
where
	T: WriteType,
{
	type WriteType = Option<T::WriteType>;
}

impl<'a, T> StaticSize for Option<T>
where
	T: StaticSize,
{
	const STATIC_SIZE: PointerType = 1 + T::STATIC_SIZE;
}

impl<'a, T> Read<'a> for Option<T>
where
	T: StaticSize + Read<'a>,
{
	type Output = Option<T::Output>;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		let is_some = bool::read(bytes, position.offset(0));
		if is_some {
			Some(T::read(bytes, position.offset(1)))
		} else {
			None
		}
	}
}

impl<T> Write for Option<T>
where
	T: StaticSize + Write,
	T::Output: Sized,
{
	type Output = Option<T::Output>;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let position = writer.position();
		match self {
			None => {
				writer.write(&0u8);
				for _ in 0..T::STATIC_SIZE {
					writer.write(&0u8);
				}
			}
			Some(value) => {
				writer.write(&1u8);
				writer.write(value);
			}
		}
		position
	}
}

impl<'a, T1, T2> ReadType<'a> for (T1, T2)
where
	T1: ReadType<'a>,
	T1::ReadType: StaticSize,
	T2: ReadType<'a>,
	T2::ReadType: StaticSize,
{
	type ReadType = (T1::ReadType, T2::ReadType);
}

impl<T1, T2> WriteType for (T1, T2)
where
	T1: WriteType,
	T2: WriteType,
{
	type WriteType = (T1::WriteType, T2::WriteType);
}

impl<'a, T1, T2> StaticSize for (T1, T2)
where
	T1: Read<'a> + StaticSize,
	T2: Read<'a> + StaticSize,
{
	const STATIC_SIZE: PointerType = T1::STATIC_SIZE + T2::STATIC_SIZE;
}

impl<'a, T1, T2> Read<'a> for (T1, T2)
where
	T1: Read<'a> + StaticSize,
	T2: Read<'a> + StaticSize,
{
	type Output = (T1::Output, T2::Output);
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		(
			T1::read(bytes, position.offset(0)),
			T2::read(bytes, position.offset(T1::STATIC_SIZE)),
		)
	}
}

impl<T1, T2> Write for (T1, T2)
where
	T1: Write + StaticSize,
	T2: Write + StaticSize,
	T1::Output: Sized,
	T2::Output: Sized,
{
	type Output = (T1::Output, T2::Output);
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let position = writer.position();
		self.0.write(writer);
		self.1.write(writer);
		position
	}
}

impl<'a, T> ReadType<'a> for Vec<T>
where
	T: 'a + ReadType<'a>,
	T::ReadType: StaticSize,
{
	type ReadType = Pointer<VecReader<'a, T::ReadType>>;
}

impl<T> WriteType for Vec<T>
where
	T: WriteType,
{
	type WriteType = Position<[T::WriteType]>;
}

impl<T> Write for Vec<T>
where
	T: Write,
	T::Output: Sized,
{
	type Output = [T::Output];
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(self.as_slice())
	}
}

impl<T> Write for [T]
where
	T: Write,
	T::Output: Sized,
{
	type Output = [T::Output];
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let position = writer.position();
		let len = self.len() as PointerType;
		writer.write(&len);
		for value in self.iter() {
			writer.write(value);
		}
		position
	}
}

impl<'a> ReadType<'a> for String {
	type ReadType = Pointer<&'a str>;
}

impl WriteType for String {
	type WriteType = Position<str>;
}

impl<'a> StaticSize for &'a str {
	const STATIC_SIZE: PointerType = size_of::<PointerType>() as PointerType;
}

impl<'a> Read<'a> for &'a str {
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		let len = PointerType::read(bytes, position.cast());
		let len = len.to_usize().unwrap();
		let position = position.offset::<str>(PointerType::STATIC_SIZE);
		let position = *position as usize;
		std::str::from_utf8(&bytes[position..position + len]).unwrap()
	}
}

impl Write for str {
	type Output = str;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let position = writer.position();
		let len: PointerType = self.len().try_into().unwrap();
		writer.write(&len);
		writer.write_raw::<str>(self.as_bytes());
		position
	}
}

impl Write for String {
	type Output = str;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		writer.write(self.as_str())
	}
}

impl<'a, K, V> ReadType<'a> for std::collections::HashMap<K, V>
where
	K: 'a + ReadType<'a>,
	K::ReadType: StaticSize,
	V: 'a + ReadType<'a>,
	V::ReadType: StaticSize,
{
	type ReadType = Pointer<VecReader<'a, (K::ReadType, V::ReadType)>>;
}

impl<K, V> WriteType for std::collections::HashMap<K, V>
where
	K: WriteType,
	V: WriteType,
{
	type WriteType = Position<[(K::WriteType, V::WriteType)]>;
}
