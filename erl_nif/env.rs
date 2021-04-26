use crate::{
	sys::{enif_raise_exception, ErlNifEnv, ERL_NIF_TERM},
	ToErlNif,
};
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Env<'a>(*mut ErlNifEnv, PhantomData<&'a ErlNifEnv>);

impl<'a> Env<'a> {
	pub fn from_raw(env: *mut ErlNifEnv) -> Env<'a> {
		Env(env, PhantomData)
	}

	pub fn raw(&self) -> *mut ErlNifEnv {
		self.0
	}

	pub fn raise_exception(&self, message: &str) -> ERL_NIF_TERM {
		let message = message
			.to_erl_nif(*self)
			.expect("failed to create exception message");
		unsafe { enif_raise_exception(self.raw(), message.raw()) }
	}
}
