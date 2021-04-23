use crate::{
	sys::{enif_get_tuple, enif_make_tuple_from_array},
	Env, Error, Result, Term,
};
use std::{mem::MaybeUninit, os::raw::c_uint};

#[derive(Clone, Copy)]
pub struct Tuple<'a>(Term<'a>);

impl<'a> Tuple<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Tuple<'a> {
		Tuple(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>, terms: impl IntoIterator<Item = Term<'a>>) -> Result<Tuple<'a>> {
		let terms = terms.into_iter().map(|term| term.raw()).collect::<Vec<_>>();
		let term =
			unsafe { enif_make_tuple_from_array(env.raw(), terms.as_ptr(), terms.len() as c_uint) };
		let term = Term::from_raw(env, term);
		let tuple = Tuple::from_term(term);
		Ok(tuple)
	}

	pub fn get(&self) -> Result<Vec<Term<'a>>> {
		let terms = unsafe {
			let mut terms = MaybeUninit::uninit();
			let mut len = MaybeUninit::uninit();
			let success = enif_get_tuple(
				self.env().raw(),
				self.term().raw(),
				len.as_mut_ptr(),
				terms.as_mut_ptr(),
			);
			if success == 0 {
				return Err(Error::message("unable to access tuple"));
			}
			let terms = terms.assume_init();
			let len = len.assume_init() as usize;
			std::slice::from_raw_parts(terms, len)
		};
		let terms = terms
			.iter()
			.map(|term| Term::from_raw(self.env(), *term))
			.collect();
		Ok(terms)
	}
}
