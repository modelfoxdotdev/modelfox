use crate::sys::ErlNifEntry;

pub struct Entry(ErlNifEntry);

unsafe impl Sync for Entry {}

impl Entry {
	pub const fn new(entry: ErlNifEntry) -> Entry {
		Entry(entry)
	}

	pub fn raw(&self) -> &ErlNifEntry {
		&self.0
	}
}
