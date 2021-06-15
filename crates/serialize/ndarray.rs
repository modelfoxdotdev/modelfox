use crate::{Pointer, PointerType, Position, Read, ReadType, StaticSize, Write, WriteType};
use ndarray::prelude::*;
use num::ToPrimitive;

impl<'a, T> ReadType<'a> for Array1<T>
where
	T: 'a + ReadType<'a>,
	T::ReadType: StaticSize + Read<'a>,
{
	type ReadType = Pointer<Array1<T::ReadType>>;
}

impl<'a, T> Read<'a> for Array1<T>
where
	T: StaticSize + Read<'a>,
{
	type Output = Array1<T::Output>;
	fn read(bytes: &'a [u8], position: crate::Position<Self>) -> Self::Output {
		let len = PointerType::read(bytes, position.cast())
			.to_usize()
			.unwrap();
		let mut position = position.offset::<T>(PointerType::STATIC_SIZE);
		let mut output = Vec::with_capacity(len);
		for _ in 0..len {
			output.push(T::read(bytes, position));
			position = position.offset::<T>(T::STATIC_SIZE);
		}
		Array1::from_vec(output)
	}
}

impl<T> WriteType for Array1<T>
where
	T: WriteType,
{
	type WriteType = Position<Array1<T::WriteType>>;
}

impl<T> Write for Array1<T>
where
	T: Write,
	T::Output: Sized,
{
	type Output = Array1<T>;
	fn write(&self, writer: &mut crate::Writer) -> crate::Position<Self::Output> {
		let position = writer.position();
		let len = self.len() as PointerType;
		writer.write(&len);
		for value in self.iter() {
			writer.write(value);
		}
		position
	}
}

impl<'a, T> ReadType<'a> for Array2<T>
where
	T: 'a + ReadType<'a>,
	T::ReadType: StaticSize + Read<'a>,
{
	type ReadType = Pointer<Array2<T::ReadType>>;
}

impl<'a, T> Read<'a> for Array2<T>
where
	T: StaticSize + Read<'a>,
{
	type Output = Array2<T::Output>;
	fn read(bytes: &'a [u8], position: crate::Position<Self>) -> Self::Output {
		let nrows = PointerType::read(bytes, position.cast())
			.to_usize()
			.unwrap();
		let position = position.offset::<PointerType>(PointerType::STATIC_SIZE);
		let ncols = PointerType::read(bytes, position.cast())
			.to_usize()
			.unwrap();
		let mut position = position.offset::<T>(PointerType::STATIC_SIZE);
		let mut output = Vec::with_capacity(nrows * ncols);
		for _ in 0..nrows * ncols {
			output.push(T::read(bytes, position));
			position = position.offset::<T>(T::STATIC_SIZE);
		}
		Array2::from_shape_vec((nrows, ncols), output).unwrap()
	}
}

impl<T> WriteType for Array2<T>
where
	T: WriteType,
{
	type WriteType = Position<Array2<T::WriteType>>;
}

impl<T> Write for Array2<T>
where
	T: Write,
	T::Output: Sized,
{
	type Output = Array2<T>;
	fn write(&self, writer: &mut crate::Writer) -> crate::Position<Self::Output> {
		let position = writer.position();
		let nrows = self.nrows() as PointerType;
		writer.write(&nrows);
		let ncols = self.ncols() as PointerType;
		writer.write(&ncols);
		for value in self.iter() {
			writer.write(value);
		}
		position
	}
}
