use crate::{Env, Term};

#[derive(Clone, Copy)]
pub struct Pid<'a>(Term<'a>);

impl<'a> Pid<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Pid<'a> {
		Pid(term)
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}
}
