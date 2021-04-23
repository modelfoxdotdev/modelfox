use crate::{DynamicStructIndexWriter, Position, Write};
use std::{collections::HashMap, convert::TryInto};

pub struct Writer {
	indexes: HashMap<DynamicStructIndexWriter, Position<DynamicStructIndexWriter>>,
	buffer: Vec<u8>,
}

impl Default for Writer {
	fn default() -> Self {
		Writer::new()
	}
}

impl Writer {
	pub fn new() -> Writer {
		Writer {
			indexes: HashMap::new(),
			buffer: Vec::new(),
		}
	}

	pub fn position<T>(&self) -> Position<T>
	where
		T: ?Sized,
	{
		Position::new(self.buffer.len().try_into().unwrap())
	}

	pub fn write_raw<T>(&mut self, bytes: &[u8]) -> Position<T::Output>
	where
		T: Write + ?Sized,
	{
		let position = self.position();
		self.buffer.extend(bytes);
		position
	}

	pub fn write<T>(&mut self, value: &T) -> Position<T::Output>
	where
		T: Write + ?Sized,
	{
		value.write(self)
	}

	pub fn add_index(
		&mut self,
		index: DynamicStructIndexWriter,
		position: Position<DynamicStructIndexWriter>,
	) {
		self.indexes.insert(index, position);
	}

	pub fn get_index(
		&self,
		index: &DynamicStructIndexWriter,
	) -> Option<&Position<DynamicStructIndexWriter>> {
		self.indexes.get(index)
	}

	pub fn into_bytes(self) -> Vec<u8> {
		self.buffer
	}
}
