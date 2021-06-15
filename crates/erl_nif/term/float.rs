use crate::{
	sys::{enif_get_double, enif_make_double},
	Env, Error, Result, Term,
};
use std::mem::MaybeUninit;

#[derive(Clone, Copy)]
pub struct Float<'a>(Term<'a>);

impl<'a> Float<'a> {
	pub(crate) fn from_term(term: Term) -> Float {
		Float(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn new(env: Env, value: f64) -> Float {
		let term = unsafe { enif_make_double(env.raw(), value) };
		let term = Term::from_raw(env, term);
		Float::from_term(term)
	}

	pub fn get(&self) -> Result<f64> {
		let mut value = MaybeUninit::uninit();
		let success =
			unsafe { enif_get_double(self.env().raw(), self.term().raw(), value.as_mut_ptr()) };
		if success == 0 {
			return Err(Error::message("unable to get f64 from float"));
		}
		let value = unsafe { value.assume_init() };
		Ok(value)
	}
}
