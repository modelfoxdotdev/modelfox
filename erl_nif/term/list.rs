use crate::{
	sys::{
		enif_get_list_cell, enif_get_list_length, enif_make_list_from_array, enif_make_string_len,
		ErlNifCharEncoding,
	},
	Env, Error, Result, Term,
};
use std::{
	mem::MaybeUninit,
	os::raw::{c_char, c_uint},
};

#[derive(Clone, Copy)]
pub struct List<'a>(Term<'a>);

impl<'a> List<'a> {
	pub(crate) fn from_term(term: Term) -> List {
		List(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(
		env: Env<'a>,
		terms: impl IntoIterator<Item = impl Into<Term<'a>>>,
	) -> Result<List<'a>> {
		let terms = terms
			.into_iter()
			.map(|term| term.into().raw())
			.collect::<Vec<_>>();
		let term =
			unsafe { enif_make_list_from_array(env.raw(), terms.as_ptr(), terms.len() as c_uint) };
		let term = Term::from_raw(env, term);
		let list = List(term);
		Ok(list)
	}

	pub fn from_string(env: Env<'a>, string: &str) -> Result<List<'a>> {
		let term = unsafe {
			enif_make_string_len(
				env.raw(),
				string.as_ptr() as *const c_char,
				string.len(),
				ErlNifCharEncoding::ERL_NIF_LATIN1,
			)
		};
		Ok(List(Term::from_raw(env, term)))
	}

	pub fn size(&self) -> Result<usize> {
		let len = unsafe {
			let mut len = MaybeUninit::uninit();
			let success =
				enif_get_list_length(self.env().raw(), self.term().raw(), len.as_mut_ptr());
			if success == 0 {
				return Err(Error::message("failed to get list length"));
			}
			len.assume_init()
		};
		Ok(len as usize)
	}

	pub fn iter(&self) -> ListIterator<'a> {
		ListIterator::new(*self)
	}
}

pub struct ListIterator<'a> {
	term: Term<'a>,
}

impl<'a> ListIterator<'a> {
	fn new(list: List<'a>) -> ListIterator<'a> {
		ListIterator { term: list.term() }
	}
}

impl<'a> Iterator for ListIterator<'a> {
	type Item = Term<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let cell = unsafe {
			let mut head = MaybeUninit::uninit();
			let mut tail = MaybeUninit::uninit();
			let success = enif_get_list_cell(
				self.term.env.raw(),
				self.term.raw(),
				head.as_mut_ptr(),
				tail.as_mut_ptr(),
			);
			if success == 0 {
				None
			} else {
				Some((head.assume_init(), tail.assume_init()))
			}
		};
		match cell {
			Some((head, tail)) => {
				self.term = Term::from_raw(self.term.env, tail);
				Some(Term::from_raw(self.term.env, head))
			}
			None => None,
		}
	}
}
