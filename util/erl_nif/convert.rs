use crate::{
	Atom, BinaryTerm, BitString, Env, Error, Float, Fun, Integer, List, Map, Pid, Port, Result,
	Term, Tuple,
};
use num::{FromPrimitive, ToPrimitive};

#[allow(clippy::wrong_self_convention, clippy::upper_case_acronyms)]
pub trait ToErlNif<'a>: 'a {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>>;
}

#[allow(clippy::wrong_self_convention, clippy::upper_case_acronyms)]
pub trait FromErlNif<'a>: 'a + Sized {
	fn from_erl_nif(term: Term<'a>) -> Result<Self>;
}

impl<'a> ToErlNif<'a> for () {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		Ok(Atom::new(env, "nil")?.term())
	}
}

impl<'a> FromErlNif<'a> for () {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		let atom = term.as_atom()?;
		if atom.get()? != "nil" {
			return Err(Error::message("term was not nil atom"));
		}
		Ok(())
	}
}

impl<'a> ToErlNif<'a> for bool {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		let atom = match self {
			true => "true",
			false => "false",
		};
		let atom = Atom::new(env, atom)?;
		Ok(atom.term())
	}
}

impl<'a> FromErlNif<'a> for bool {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		let atom = term.as_atom()?;
		match atom.get()?.as_str() {
			"true" => Ok(true),
			"false" => Ok(false),
			_ => Err(Error::message("term was not :true or :false")),
		}
	}
}

macro_rules! impl_to_from_for_integer_type {
	($ty:ty) => {
		impl<'a> ToErlNif<'a> for $ty {
			fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
				let value =
					<$ty>::to_i64(&self).ok_or_else(|| Error::message("integer out of bounds"))?;
				let integer = Integer::new(env, value);
				Ok(integer.term())
			}
		}
		impl<'a> FromErlNif<'a> for $ty {
			fn from_erl_nif(term: Term<'a>) -> Result<Self> {
				let integer = term.as_integer()?;
				let value = integer.get()?;
				let value = <$ty>::from_i64(value)
					.ok_or_else(|| Error::message("integer out of bounds"))?;
				Ok(value)
			}
		}
	};
}

impl_to_from_for_integer_type!(usize);
impl_to_from_for_integer_type!(u8);
impl_to_from_for_integer_type!(u16);
impl_to_from_for_integer_type!(u32);
impl_to_from_for_integer_type!(u64);
impl_to_from_for_integer_type!(isize);
impl_to_from_for_integer_type!(i8);
impl_to_from_for_integer_type!(i16);
impl_to_from_for_integer_type!(i32);
impl_to_from_for_integer_type!(i64);

macro_rules! impl_to_from_for_float_type {
	($ty:ty) => {
		impl<'a> ToErlNif<'a> for $ty {
			fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
				let value =
					<$ty>::to_f64(&self).ok_or_else(|| Error::message("float out of bounds"))?;
				let float = Float::new(env, value);
				Ok(float.term())
			}
		}
		impl<'a> FromErlNif<'a> for $ty {
			fn from_erl_nif(term: Term<'a>) -> Result<Self> {
				let float = term.as_float()?;
				let value = float.get()?;
				let value =
					<$ty>::from_f64(value).ok_or_else(|| Error::message("float out of bounds"))?;
				Ok(value)
			}
		}
	};
}

impl_to_from_for_float_type!(f32);
impl_to_from_for_float_type!(f64);

impl<'a> ToErlNif<'a> for char {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		self.to_string().to_erl_nif(env)
	}
}

impl<'a> FromErlNif<'a> for char {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		let string = String::from_erl_nif(term)?;
		let mut chars = string.chars();
		let c = if let Some(c) = chars.next() {
			c
		} else {
			return Err(Error::message("expected one character"));
		};
		if chars.next().is_some() {
			return Err(Error::message("expected one character"));
		}
		Ok(c)
	}
}

impl<'a> ToErlNif<'a> for &'a str {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		Ok(BinaryTerm::from_str(env, self)?.term())
	}
}

impl<'a> ToErlNif<'a> for std::string::String {
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		Ok(BinaryTerm::from_str(env, &self)?.term())
	}
}

impl<'a> FromErlNif<'a> for std::string::String {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		if term.is_binary() {
			let binary = term.as_binary()?;
			let bytes = binary.get()?.to_owned();
			let string = String::from_utf8(bytes)
				.map_err(|_| Error::message("The string was not valid UTF-8"))?;
			Ok(string)
		} else if term.is_atom() {
			let atom = term.as_atom()?;
			let string = atom.get()?;
			Ok(string)
		} else {
			Err(Error::bad_arg())
		}
	}
}

impl<'a, T> ToErlNif<'a> for Option<T>
where
	T: ToErlNif<'a>,
{
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		match self {
			None => Ok(Atom::new(env, "nil")?.term()),
			Some(term) => Ok(term.to_erl_nif(env)?),
		}
	}
}

impl<'a, T> FromErlNif<'a> for Option<T>
where
	T: FromErlNif<'a>,
{
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		let is_nil = if term.is_atom() {
			let atom = term.as_atom()?;
			let atom = atom.get()?;
			atom.as_str() == "nil"
		} else {
			false
		};
		if is_nil {
			Ok(None)
		} else {
			Ok(Some(T::from_erl_nif(term)?))
		}
	}
}

impl<'a, T> ToErlNif<'a> for Vec<T>
where
	T: ToErlNif<'a>,
{
	fn to_erl_nif(self, env: Env<'a>) -> Result<Term<'a>> {
		let terms = self
			.into_iter()
			.map(|value| value.to_erl_nif(env))
			.collect::<Result<Vec<_>>>()?;
		let list = List::new(env, terms)?;
		Ok(list.term())
	}
}

impl<'a, T> FromErlNif<'a> for Vec<T>
where
	T: FromErlNif<'a>,
{
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_list()?
			.iter()
			.map(T::from_erl_nif)
			.collect::<Result<_>>()
	}
}

impl<'a> ToErlNif<'a> for Term<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self)
	}
}

impl<'a> FromErlNif<'a> for Term<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		Ok(term)
	}
}

impl<'a> ToErlNif<'a> for Atom<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Atom<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_atom()
	}
}

impl<'a> ToErlNif<'a> for BinaryTerm<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for BinaryTerm<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_binary()
	}
}

impl<'a> ToErlNif<'a> for BitString<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for BitString<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_bitstring()
	}
}

impl<'a> ToErlNif<'a> for Float<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Float<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_float()
	}
}

impl<'a> ToErlNif<'a> for Fun<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Fun<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_fun()
	}
}

impl<'a> ToErlNif<'a> for Integer<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Integer<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_integer()
	}
}

impl<'a> ToErlNif<'a> for List<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for List<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_list()
	}
}

impl<'a> ToErlNif<'a> for Map<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Map<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_map()
	}
}

impl<'a> ToErlNif<'a> for Pid<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Pid<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_pid()
	}
}

impl<'a> ToErlNif<'a> for Port<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Port<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_port()
	}
}

// impl<T> ToErlNif for ResourceTerm<T> {
// 	fn to_erl_nif(self, _env: Env) -> Result<Term> {
// 		Ok(self.term())
// 	}
// }

// impl<T> FromErlNif for ResourceTerm<T> {
// 	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
// 		term.as_resource()
// 	}
// }

impl<'a> ToErlNif<'a> for Tuple<'a> {
	fn to_erl_nif(self, _env: Env<'a>) -> Result<Term<'a>> {
		Ok(self.term())
	}
}

impl<'a> FromErlNif<'a> for Tuple<'a> {
	fn from_erl_nif(term: Term<'a>) -> Result<Self> {
		term.as_tuple()
	}
}
