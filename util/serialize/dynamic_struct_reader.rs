use crate::{
	DynamicStructIndexFieldId, DynamicStructIndexPointer, DynamicStructIndexReader,
	DynamicStructIndexType, PointerType, Position, Read,
};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct DynamicStructReader<'a, I> {
	bytes: &'a [u8],
	position: Position<Self>,
	marker: PhantomData<I>,
}

impl<'a, I> DynamicStructReader<'a, I>
where
	I: DynamicStructIndexType,
{
	pub fn new(bytes: &'a [u8], position: Position<Self>) -> DynamicStructReader<'a, I> {
		DynamicStructReader {
			bytes,
			position,
			marker: PhantomData,
		}
	}

	pub fn get_field_value<T>(&self, field_id: DynamicStructIndexFieldId<I>) -> Option<T::Output>
	where
		T: Read<'a>,
	{
		let field_offset: PointerType = self.index().get_field_offset(field_id).0.into();
		if field_offset != 0 {
			let field_position = self.position.offset(field_offset);
			Some(T::read(self.bytes, field_position))
		} else {
			None
		}
	}

	fn index(&self) -> DynamicStructIndexReader<'a, I> {
		let index_pointer_position = self.position.offset(0);
		let index_pointer = DynamicStructIndexPointer::read(self.bytes, index_pointer_position);
		let index_position = Position::new(*self.position - *index_pointer);
		DynamicStructIndexReader::new(self.bytes, index_position)
	}
}

impl<'a, I> Read<'a> for DynamicStructReader<'a, I>
where
	I: DynamicStructIndexType,
{
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		DynamicStructReader::new(bytes, position)
	}
}
