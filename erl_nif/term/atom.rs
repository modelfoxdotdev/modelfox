use crate::{
	sys::{
		enif_get_atom, enif_get_atom_length, enif_make_atom_len, enif_make_existing_atom_len,
		ErlNifCharEncoding,
	},
	Env, Error, Result, Term,
};
use std::{mem::MaybeUninit, os::raw::c_char};

#[derive(Clone, Copy)]
pub struct Atom<'a>(Term<'a>);

impl<'a> Atom<'a> {
	pub(crate) fn from_term(term: Term) -> Atom {
		Atom(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>, name: &str) -> Result<Atom<'a>> {
		let term = unsafe {
			let mut term = MaybeUninit::uninit();
			let success = enif_make_existing_atom_len(
				env.raw(),
				name.as_ptr() as *const c_char,
				name.len(),
				term.as_mut_ptr(),
				ErlNifCharEncoding::ERL_NIF_LATIN1,
			);
			if success == 0 {
				*term.as_mut_ptr() =
					enif_make_atom_len(env.raw(), name.as_ptr() as *const c_char, name.len())
			}
			term.assume_init()
		};
		let term = Term::from_raw(env, term);
		Ok(Atom::from_term(term))
	}

	pub fn existing(env: Env<'a>, name: &str) -> Result<Atom<'a>> {
		let term = unsafe {
			let mut term = MaybeUninit::uninit();
			let success = enif_make_existing_atom_len(
				env.raw(),
				name.as_ptr() as *const c_char,
				name.len(),
				term.as_mut_ptr(),
				ErlNifCharEncoding::ERL_NIF_LATIN1,
			);
			if success == 0 {
				return Err(Error::message(format!(
					"there is no existing atom \"{}\"",
					name
				)));
			}
			term.assume_init()
		};
		let term = Term::from_raw(env, term);
		Ok(Atom::from_term(term))
	}

	pub fn get(&self) -> Result<String> {
		let mut len = MaybeUninit::uninit();
		let success = unsafe {
			enif_get_atom_length(
				self.env().raw(),
				self.term().raw(),
				len.as_mut_ptr(),
				ErlNifCharEncoding::ERL_NIF_LATIN1,
			)
		};
		if success == 0 {
			return Err(Error::message("failed to get atom length"));
		}
		let len = unsafe { len.assume_init() };
		let mut bytes: Vec<u8> = Vec::with_capacity(len as usize + 1);
		let len_written = unsafe {
			enif_get_atom(
				self.env().raw(),
				self.term().raw(),
				bytes.as_mut_ptr() as *mut c_char,
				len + 1,
				ErlNifCharEncoding::ERL_NIF_LATIN1,
			)
		};
		if len_written as usize != len as usize + 1 {
			return Err(Error::message("failed to read atom"));
		}
		unsafe { bytes.set_len(len as usize) };
		let string = unsafe { std::string::String::from_utf8_unchecked(bytes) };
		Ok(string)
	}
}
