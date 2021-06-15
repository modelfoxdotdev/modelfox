use crate::{Position, Read};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct VariantReader<'a, T>
where
	T: Read<'a>,
{
	bytes: &'a [u8],
	position: Position<Self>,
	marker: PhantomData<T>,
}

impl<'a, T> VariantReader<'a, T>
where
	T: Read<'a>,
{
	pub fn new(bytes: &'a [u8], position: Position<Self>) -> VariantReader<'a, T> {
		VariantReader {
			bytes,
			position,
			marker: PhantomData,
		}
	}

	pub fn read(&self) -> T::Output {
		T::read(self.bytes, self.position.cast())
	}
}
