use crate::{Env, Term};

pub struct Fun<'a>(Term<'a>);

impl<'a> Fun<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Fun<'a> {
		Fun(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
