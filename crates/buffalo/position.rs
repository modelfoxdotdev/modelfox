use crate::{PointerType, StaticSize, Write, Writer};
use derive_deref::Deref;
use std::{marker::PhantomData, mem::size_of};

#[derive(Debug, Deref)]
pub struct Position<T>(PointerType, PhantomData<T>)
where
	T: ?Sized;

impl<T> Clone for Position<T>
where
	T: ?Sized,
{
	fn clone(&self) -> Self {
		Position(self.0, PhantomData)
	}
}

impl<T> Copy for Position<T> where T: ?Sized {}

impl<T> Position<T>
where
	T: ?Sized,
{
	pub fn new(value: PointerType) -> Position<T> {
		Position(value, PhantomData)
	}

	pub fn cast<U>(&self) -> Position<U> {
		Position::new(self.0)
	}

	pub fn offset<U: ?Sized>(&self, offset: PointerType) -> Position<U> {
		Position::new(self.0 + offset)
	}
}

impl<T> StaticSize for Position<T>
where
	T: ?Sized,
{
	const STATIC_SIZE: PointerType = size_of::<PointerType>() as PointerType;
}

impl<T> Write for Position<T>
where
	T: ?Sized,
{
	type Output = Position<T>;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let src = writer.position::<Position<()>>();
		let dst = self;
		let offset = *src - **dst;
		writer.write(&offset).cast()
	}
}
