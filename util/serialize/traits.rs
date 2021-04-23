use crate::{Position, Writer};

pub type PointerType = u64;

pub trait StaticSize: Sized {
	const STATIC_SIZE: PointerType;
}

pub trait Read<'a>: Sized {
	type Output;
	fn read(bytes: &'a [u8], position: Position<Self>) -> Self::Output;
}

pub trait Write {
	type Output: ?Sized;
	fn write(&self, writer: &mut Writer) -> Position<Self::Output>;
}

pub trait ReadType<'a> {
	type ReadType: Read<'a>;
}

pub trait WriteType {
	type WriteType;
}
