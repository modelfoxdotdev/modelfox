use crate::{PointerType, Position, Read, StaticSize, Write, Writer};
use derive_deref::Deref;
use std::{marker::PhantomData, mem::size_of};

/// A `Pointer` holds a relative position.
#[derive(Debug, Deref)]
pub struct Pointer<T>(PointerType, PhantomData<T>)
where
	T: ?Sized;

impl<T> Clone for Pointer<T> {
	fn clone(&self) -> Self {
		Pointer(self.0, PhantomData)
	}
}

impl<T> std::marker::Copy for Pointer<T> {}

impl<T> Pointer<T>
where
	T: ?Sized,
{
	pub fn new(value: PointerType) -> Pointer<T> {
		Pointer(value, PhantomData)
	}
}

impl<'a, T> StaticSize for Pointer<T>
where
	T: ?Sized,
{
	const STATIC_SIZE: PointerType = size_of::<PointerType>() as PointerType;
}

impl<'a, T> Read<'a> for Pointer<T>
where
	T: Read<'a> + ?Sized,
{
	type Output = T::Output;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		let pointer_position = position.cast();
		let pointer = PointerType::read(bytes, pointer_position);
		let position = position.checked_sub(pointer).unwrap();
		let position = Position::new(position);
		T::read(bytes, position)
	}
}

impl<T> Write for Pointer<T> {
	type Output = Pointer<T>;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		self.0.write(writer).cast()
	}
}
