use crate::{PointerType, Position, Read};

#[derive(Clone, Copy, Debug)]
pub struct StaticStructReader<'a> {
	bytes: &'a [u8],
	position: Position<Self>,
}

impl<'a> StaticStructReader<'a> {
	pub fn new(bytes: &'a [u8], position: Position<Self>) -> StaticStructReader {
		StaticStructReader { bytes, position }
	}

	pub fn get_field_value<T>(&self, field_offset: PointerType) -> Option<T::Output>
	where
		T: Read<'a>,
	{
		let field_position = self.position.offset(field_offset);
		Some(T::read(self.bytes, field_position))
	}
}

impl<'a> Read<'a> for StaticStructReader<'a> {
	type Output = Self;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output {
		Self::new(bytes, position)
	}
}
