use crate::{
	sys::{
		enif_alloc_binary, enif_inspect_binary, enif_make_binary, enif_release_binary, ErlNifBinary,
	},
	Env, Error, Result, Term,
};
use std::mem::MaybeUninit;

pub struct Binary(ErlNifBinary);

impl Drop for Binary {
	fn drop(&mut self) {
		unsafe { enif_release_binary(&mut self.0) }
	}
}

impl Binary {
	pub fn new(len: usize) -> Result<Binary> {
		let binary = unsafe {
			let mut binary = MaybeUninit::uninit();
			let success = enif_alloc_binary(len, binary.as_mut_ptr());
			if success == 0 {
				return Err(Error::message("failed to allocate binary"));
			}
			binary.assume_init()
		};
		Ok(Binary(binary))
	}

	pub fn get(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.0.data, self.0.size) }
	}

	pub fn get_mut(&mut self) -> &mut [u8] {
		unsafe { std::slice::from_raw_parts_mut(self.0.data, self.0.size) }
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Binary> {
		let mut binary = Binary::new(bytes.len())?;
		binary.get_mut().copy_from_slice(bytes);
		Ok(binary)
	}
}

pub struct BinaryTerm<'a> {
	term: Term<'a>,
	binary: Binary,
}

impl<'a> BinaryTerm<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Result<BinaryTerm<'a>> {
		let binary = unsafe {
			let mut binary = MaybeUninit::uninit();
			let success = enif_inspect_binary(term.env().raw(), term.raw(), binary.as_mut_ptr());
			if success == 0 {
				return Err(Error::message("could not inspect binary"));
			}
			binary.assume_init()
		};
		let binary = Binary(binary);
		Ok(BinaryTerm { term, binary })
	}

	pub fn term(&self) -> Term<'a> {
		self.term
	}

	pub fn env(&self) -> Env<'a> {
		self.term.env()
	}

	pub fn new(env: Env<'a>, binary: Binary) -> Result<BinaryTerm<'a>> {
		let mut binary = std::mem::ManuallyDrop::new(binary);
		let term = unsafe { enif_make_binary(env.raw(), &mut binary.0) };
		let term = Term::from_raw(env, term);
		let term = BinaryTerm::from_term(term)?;
		Ok(term)
	}

	pub fn from_bytes(env: Env<'a>, bytes: &[u8]) -> Result<BinaryTerm<'a>> {
		let binary = Binary::from_bytes(bytes)?;
		let term = BinaryTerm::new(env, binary)?;
		Ok(term)
	}

	pub fn from_str(env: Env<'a>, string: &str) -> Result<BinaryTerm<'a>> {
		let binary = Binary::from_bytes(string.as_bytes())?;
		let term = BinaryTerm::new(env, binary)?;
		Ok(term)
	}

	pub fn binary(&self) -> &Binary {
		&self.binary
	}

	pub fn get(&self) -> Result<&[u8]> {
		Ok(self.binary.get())
	}

	pub fn get_mut(&mut self) -> Result<&mut [u8]> {
		Ok(self.binary.get_mut())
	}
}
