use crate::{Env, Term};

#[derive(Clone, Copy)]
pub struct Port<'a>(Term<'a>);

impl<'a> Port<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Port<'a> {
		Port(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
