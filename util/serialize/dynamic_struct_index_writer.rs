use crate::{DynamicStructIndexType, Position, Write, Writer};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicStructIndexWriter {
	U8(DynamicStructIndexWriterI<u8>),
	U16(DynamicStructIndexWriterI<u16>),
}

impl Write for DynamicStructIndexWriter {
	type Output = DynamicStructIndexWriter;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		match self {
			DynamicStructIndexWriter::U8(s) => writer.write(s).cast(),
			DynamicStructIndexWriter::U16(s) => writer.write(s).cast(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynamicStructIndexWriterI<I>(Vec<Option<I>>)
where
	I: DynamicStructIndexType;

impl<I> DynamicStructIndexWriterI<I>
where
	I: DynamicStructIndexType,
{
	pub fn new(field_count: I) -> DynamicStructIndexWriterI<I> {
		DynamicStructIndexWriterI(vec![None; field_count.to_usize().unwrap()])
	}

	pub fn set_field_offset(&mut self, field_id: I, offset: I) {
		self.0.splice(
			field_id.to_usize().unwrap()..field_id.to_usize().unwrap() + 1,
			vec![Some(offset)],
		);
	}
}

impl<I> Write for DynamicStructIndexWriterI<I>
where
	I: DynamicStructIndexType,
{
	type Output = DynamicStructIndexWriterI<I>;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output> {
		let position = writer.position();
		let field_count = I::from_usize(self.0.len()).unwrap();
		writer.write(&field_count);
		for field_offset in self.0.iter() {
			writer.write(&field_offset.unwrap_or_else(I::zero));
		}
		position
	}
}
