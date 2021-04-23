use crate::{PointerType, Position, Read, StaticSize};
use num::ToPrimitive;
use std::{convert::TryInto, marker::PhantomData};

#[derive(Clone, Copy, Debug)]
pub struct VecReader<'a, T>
where
	T: StaticSize + Read<'a>,
{
	bytes: &'a [u8],
	position: Position<Self>,
	marker: PhantomData<T>,
}

impl<'a, T> VecReader<'a, T>
where
	T: StaticSize + Read<'a>,
{
	pub fn new(bytes: &'a [u8], position: Position<Self>) -> VecReader<T> {
		VecReader {
			bytes,
			position,
			marker: PhantomData,
		}
	}

	pub fn len(&self) -> usize {
		let len_position = self.position.offset(0);
		PointerType::read(self.bytes, len_position)
			.to_usize()
			.unwrap()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn get(&self, index: usize) -> Option<T::Output> {
		if index < self.len() {
			let index: PointerType = index.try_into().unwrap();
			let offset = PointerType::STATIC_SIZE + T::STATIC_SIZE * index;
			Some(T::read(self.bytes, self.position.offset(offset)))
		} else {
			None
		}
	}

	pub fn iter(self) -> VecReaderIterator<'a, T> {
		VecReaderIterator::new(self)
	}
}

impl<'a, T> Read<'a> for VecReader<'a, T>
where
	T: 'a + StaticSize + Read<'a>,
{
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		VecReader::new(bytes, position)
	}
}

pub struct VecReaderIterator<'a, T>
where
	T: StaticSize + Read<'a>,
{
	vec_reader: VecReader<'a, T>,
	i: usize,
}

impl<'a, T> VecReaderIterator<'a, T>
where
	T: StaticSize + Read<'a>,
{
	pub fn new(vec_reader: VecReader<'a, T>) -> VecReaderIterator<'a, T> {
		VecReaderIterator { vec_reader, i: 0 }
	}
}

impl<'a, T> Iterator for VecReaderIterator<'a, T>
where
	T: StaticSize + Read<'a>,
{
	type Item = T::Output;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.vec_reader.len() {
			let item = self.vec_reader.get(self.i).unwrap();
			self.i += 1;
			Some(item)
		} else {
			None
		}
	}
}
