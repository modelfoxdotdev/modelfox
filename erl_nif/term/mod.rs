use self::resource::ResourceType;
use crate::{
	sys::{enif_is_binary, enif_term_type, ErlNifTermType, ERL_NIF_TERM},
	Env, Error, Result,
};

pub mod atom;
pub mod binary;
pub mod bitstring;
pub mod float;
pub mod fun;
pub mod integer;
pub mod list;
pub mod map;
pub mod pid;
pub mod port;
pub mod resource;
pub mod tuple;

pub use self::{
	atom::Atom, binary::BinaryTerm, bitstring::BitString, float::Float, fun::Fun, integer::Integer,
	list::List, map::Map, pid::Pid, port::Port, resource::ResourceTerm, tuple::Tuple,
};

#[derive(Clone, Copy)]
pub struct Term<'a> {
	env: Env<'a>,
	term: ERL_NIF_TERM,
}

impl<'a> Term<'a> {
	pub fn from_raw(env: Env<'a>, term: ERL_NIF_TERM) -> Term<'a> {
		Term { env, term }
	}

	pub fn env(&self) -> Env<'a> {
		self.env
	}

	pub fn raw(&self) -> ERL_NIF_TERM {
		self.term
	}

	pub fn term_type(&self) -> ErlNifTermType {
		unsafe { enif_term_type(self.env.raw(), self.term) }
	}

	pub fn is_atom(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_ATOM
	}

	pub fn as_atom(&self) -> Result<Atom<'a>> {
		if self.is_atom() {
			Ok(Atom::from_term(*self))
		} else {
			Err(Error::message("could not cast term to atom"))
		}
	}

	pub fn is_binary(&self) -> bool {
		unsafe { enif_is_binary(self.env().raw(), self.raw()) != 0 }
	}

	pub fn as_binary(&self) -> Result<BinaryTerm<'a>> {
		if self.is_binary() {
			Ok(BinaryTerm::from_term(*self)?)
		} else {
			Err(Error::message("could not cast term to binary"))
		}
	}

	pub fn is_bitstring(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_BITSTRING
	}

	pub fn as_bitstring(&self) -> Result<BitString<'a>> {
		if self.is_bitstring() {
			Ok(BitString::from_term(*self))
		} else {
			Err(Error::message("could not cast term to bitstring"))
		}
	}

	pub fn is_float(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_FLOAT
	}

	pub fn as_float(&self) -> Result<Float<'a>> {
		if self.is_float() {
			Ok(Float::from_term(*self))
		} else {
			Err(Error::message("could not cast term to float"))
		}
	}

	pub fn is_fun(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_FUN
	}

	pub fn as_fun(&self) -> Result<Fun<'a>> {
		if self.is_fun() {
			Ok(Fun::from_term(*self))
		} else {
			Err(Error::message("could not cast term to fun"))
		}
	}

	pub fn is_integer(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_INTEGER
	}

	pub fn as_integer(&self) -> Result<Integer<'a>> {
		if self.is_integer() {
			Ok(Integer::from_term(*self))
		} else {
			Err(Error::message("could not cast term to integer"))
		}
	}

	pub fn is_list(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_LIST
	}

	pub fn as_list(&self) -> Result<List<'a>> {
		if self.is_list() {
			Ok(List::from_term(*self))
		} else {
			Err(Error::message("could not cast term to list"))
		}
	}

	pub fn is_map(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_MAP
	}

	pub fn as_map(&self) -> Result<Map<'a>> {
		if self.is_map() {
			Ok(Map::from_term(*self))
		} else {
			Err(Error::message("could not cast term to map"))
		}
	}

	pub fn is_pid(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_PID
	}

	pub fn as_pid(&self) -> Result<Pid<'a>> {
		if self.is_pid() {
			Ok(Pid::from_term(*self))
		} else {
			Err(Error::message("could not cast term to pid"))
		}
	}

	pub fn is_port(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_PORT
	}

	pub fn as_port(&self) -> Result<Port<'a>> {
		if self.is_port() {
			Ok(Port::from_term(*self))
		} else {
			Err(Error::message("could not cast term to port"))
		}
	}

	pub fn is_reference(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_REFERENCE
	}

	pub fn as_resource<T>(&self, resource_type: ResourceType<T>) -> Result<ResourceTerm<T>> {
		if self.is_reference() {
			Ok(ResourceTerm::from_term(*self, resource_type)?)
		} else {
			Err(Error::message("could not cast term to resource"))
		}
	}

	pub fn is_tuple(&self) -> bool {
		self.term_type() == ErlNifTermType::ERL_NIF_TERM_TYPE_TUPLE
	}

	pub fn as_tuple(&self) -> Result<Tuple<'a>> {
		if self.is_tuple() {
			Ok(Tuple::from_term(*self))
		} else {
			Err(Error::message("could not cast term to tuple"))
		}
	}
}

impl<'a> From<Atom<'a>> for Term<'a> {
	fn from(value: Atom<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<BinaryTerm<'a>> for Term<'a> {
	fn from(value: BinaryTerm<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<BitString<'a>> for Term<'a> {
	fn from(value: BitString<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Float<'a>> for Term<'a> {
	fn from(value: Float<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Fun<'a>> for Term<'a> {
	fn from(value: Fun<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Integer<'a>> for Term<'a> {
	fn from(value: Integer<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<List<'a>> for Term<'a> {
	fn from(value: List<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Map<'a>> for Term<'a> {
	fn from(value: Map<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Pid<'a>> for Term<'a> {
	fn from(value: Pid<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Port<'a>> for Term<'a> {
	fn from(value: Port<'a>) -> Term<'a> {
		value.term()
	}
}

impl<'a, T: 'a> From<ResourceTerm<'a, T>> for Term<'a> {
	fn from(value: ResourceTerm<'a, T>) -> Term<'a> {
		value.term()
	}
}

impl<'a> From<Tuple<'a>> for Term<'a> {
	fn from(value: Tuple<'a>) -> Term<'a> {
		value.term()
	}
}
