use crate::{Env, Term};

#[derive(Clone, Copy)]
pub struct BitString<'a>(Term<'a>);

impl<'a> BitString<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> BitString<'a> {
		BitString(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
