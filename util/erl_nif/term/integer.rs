use crate::{
	sys::{enif_get_int64, enif_make_int64},
	Env, Error, Result, Term,
};
use std::mem::MaybeUninit;

#[derive(Clone, Copy)]
pub struct Integer<'a>(Term<'a>);

impl<'a> Integer<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Integer<'a> {
		Integer(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env<'a>, value: i64) -> Integer<'a> {
		let term = unsafe { enif_make_int64(env.raw(), value) };
		let term = Term::from_raw(env, term);
		Integer::from_term(term)
	}

	pub fn get(&self) -> Result<i64> {
		let mut value = MaybeUninit::uninit();
		let success =
			unsafe { enif_get_int64(self.env().raw(), self.term().raw(), value.as_mut_ptr()) };
		if success == 0 {
			return Err(Error::message("unable to get ulong from integer"));
		}
		let value = unsafe { value.assume_init() };
		Ok(value)
	}
}
