use std::convert::TryInto;

#[cfg(feature = "bitvec_0_19")]
mod bitvec;
mod dynamic_struct_index_reader;
mod dynamic_struct_index_writer;
mod dynamic_struct_reader;
#[cfg(feature = "ndarray_0_15")]
mod ndarray;
mod pointer;
mod position;
mod static_struct_reader;
mod traits;
mod types;
mod variant_reader;
mod vec_reader;
mod writer;

pub use self::{
	dynamic_struct_index_reader::{
		DynamicStructIndexFieldCount, DynamicStructIndexFieldId, DynamicStructIndexFieldOffset,
		DynamicStructIndexPointer, DynamicStructIndexReader, DynamicStructIndexType,
	},
	dynamic_struct_index_writer::{DynamicStructIndexWriter, DynamicStructIndexWriterI},
	dynamic_struct_reader::DynamicStructReader,
	pointer::Pointer,
	position::Position,
	static_struct_reader::StaticStructReader,
	traits::{PointerType, Read, ReadType, StaticSize, Write, WriteType},
	variant_reader::VariantReader,
	vec_reader::VecReader,
	writer::Writer,
};
pub use tangram_serialize_macro::{Read, Write};

pub fn read<'a, T>(bytes: &'a [u8]) -> T::Output
where
	T: Read<'a>,
{
	let position = bytes.len() - std::mem::size_of::<PointerType>();
	let position = position.try_into().unwrap();
	let position = Position::new(position);
	<Pointer<T>>::read(&bytes, position)
}

#[cfg(test)]
mod test {
	use crate as tangram_serialize;

	#[derive(tangram_serialize::Read, tangram_serialize::Write)]
	#[tangram_serialize(size = "dynamic")]
	struct AddressBook {
		#[tangram_serialize(id = 0, required)]
		pub contacts: Vec<Contact>,
	}

	#[derive(tangram_serialize::Read, tangram_serialize::Write)]
	#[tangram_serialize(size = "dynamic")]
	struct Contact {
		#[tangram_serialize(id = 0, required)]
		pub age: u16,
		#[tangram_serialize(id = 1, required)]
		pub name: String,
		#[tangram_serialize(id = 2, required)]
		pub phone_numbers: Option<Vec<PhoneNumber>>,
	}

	#[derive(tangram_serialize::Read, tangram_serialize::Write)]
	#[tangram_serialize(size = "static", value_size = 8)]
	enum PhoneNumber {
		#[allow(unused)]
		#[tangram_serialize(id = 0)]
		Home(String),
		#[allow(unused)]
		#[tangram_serialize(id = 1)]
		Mobile(String),
	}

	#[test]
	fn test_address_book() {
		let mut writer = tangram_serialize::Writer::new();
		let name = writer.write("John Doe");
		let home = writer.write("1231231234");
		let mobile = writer.write("4564564567");
		let phone_numbers = writer.write(&vec![
			PhoneNumberWriter::Home(home),
			PhoneNumberWriter::Mobile(mobile),
		]);
		let contact = writer.write(&ContactWriter {
			age: 28,
			name,
			phone_numbers: Some(phone_numbers),
		});
		let contacts = writer.write(&vec![contact]);
		let address_book = writer.write(&AddressBookWriter { contacts });
		writer.write(&address_book);
		let bytes = writer.into_bytes();
		let address_book = tangram_serialize::read::<AddressBookReader>(&bytes);
		let contact = address_book.contacts().get(0).unwrap();
		assert_eq!(contact.name(), "John Doe");
		assert_eq!(contact.age(), 28);
		let phone_numbers = contact.phone_numbers();
		let home = phone_numbers.unwrap().get(0).unwrap();
		assert_eq!(home.as_home().unwrap(), "1231231234");
		let mobile = phone_numbers.unwrap().get(1).unwrap();
		assert_eq!(mobile.as_mobile().unwrap(), "4564564567");
	}
}
