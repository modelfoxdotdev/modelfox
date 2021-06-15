use crate::{
	sys::{
		enif_get_map_size, enif_get_map_value, enif_make_map_put, enif_make_map_remove,
		enif_make_new_map, enif_map_iterator_create, enif_map_iterator_destroy,
		enif_map_iterator_get_pair, enif_map_iterator_next, ErlNifMapIterator,
		ErlNifMapIteratorEntry, ERL_NIF_TERM,
	},
	Env, Error, Result, Term,
};
use std::mem::MaybeUninit;

#[derive(Clone, Copy)]
pub struct Map<'a>(Term<'a>);

impl<'a> Map<'a> {
	pub(crate) fn from_term(term: Term<'a>) -> Map<'a> {
		Map(term)
	}

	pub fn env(&self) -> Env<'a> {
		self.0.env()
	}

	pub fn term(&self) -> Term<'a> {
		self.0
	}

	pub fn raw(&self) -> ERL_NIF_TERM {
		self.term().raw()
	}

	pub fn new(env: Env<'a>) -> Result<Map<'a>> {
		let term = unsafe { enif_make_new_map(env.raw()) };
		Ok(Map(Term::from_raw(env, term)))
	}

	pub fn size(&self) -> Result<usize> {
		let len = unsafe {
			let mut len = MaybeUninit::uninit();
			let success = enif_get_map_size(self.env().raw(), self.term().raw(), len.as_mut_ptr());
			if success == 0 {
				return Err(Error::message("unable to get map size"));
			}
			len.assume_init()
		};
		Ok(len as usize)
	}

	pub fn iter(&self) -> Result<MapIterator<'a>> {
		MapIterator::new(*self)
	}

	pub fn get(&self, key: impl Into<Term<'a>>) -> Result<Term<'a>> {
		let term = unsafe {
			let mut term = MaybeUninit::uninit();
			let success = enif_get_map_value(
				self.env().raw(),
				self.term().raw(),
				key.into().raw(),
				term.as_mut_ptr(),
			);
			if success == 0 {
				return Err(Error::message("unable to get key in map"));
			}
			term.assume_init()
		};
		Ok(Term::from_raw(self.env(), term))
	}

	pub fn set(&mut self, key: impl Into<Term<'a>>, value: impl Into<Term<'a>>) -> Result<()> {
		let term = unsafe {
			let mut term = MaybeUninit::uninit();
			let success = enif_make_map_put(
				self.env().raw(),
				self.term().raw(),
				key.into().raw(),
				value.into().raw(),
				term.as_mut_ptr(),
			);
			if success == 0 {
				return Err(Error::message("unable to set key in map"));
			}
			term.assume_init()
		};
		self.0 = Term::from_raw(self.env(), term);
		Ok(())
	}

	pub fn remove(&mut self, key: impl Into<Term<'a>>) -> Result<()> {
		let term = unsafe {
			let mut term = MaybeUninit::uninit();
			let success = enif_make_map_remove(
				self.env().raw(),
				self.term().raw(),
				key.into().raw(),
				term.as_mut_ptr(),
			);
			if success == 0 {
				return Err(Error::bad_arg());
			}
			term.assume_init()
		};
		self.0 = Term::from_raw(self.env(), term);
		Ok(())
	}
}

pub struct MapIterator<'a> {
	env: Env<'a>,
	iterator: ErlNifMapIterator,
}

impl<'a> MapIterator<'a> {
	fn new(map: Map) -> Result<MapIterator> {
		let env = map.env();
		let iterator = unsafe {
			let mut iterator = MaybeUninit::uninit();
			let success = enif_map_iterator_create(
				env.raw(),
				map.raw(),
				iterator.as_mut_ptr(),
				ErlNifMapIteratorEntry::ERL_NIF_MAP_ITERATOR_FIRST,
			);
			if success == 0 {
				return Err(Error::bad_arg());
			}
			iterator.assume_init()
		};
		Ok(MapIterator { env, iterator })
	}
}

impl<'a> Drop for MapIterator<'a> {
	fn drop(&mut self) {
		unsafe { enif_map_iterator_destroy(self.env.raw(), &mut self.iterator) };
	}
}

impl<'a> Iterator for MapIterator<'a> {
	type Item = (Term<'a>, Term<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		unsafe {
			let mut key = MaybeUninit::uninit();
			let mut value = MaybeUninit::uninit();
			let success = enif_map_iterator_get_pair(
				self.env.raw(),
				&mut self.iterator,
				key.as_mut_ptr(),
				value.as_mut_ptr(),
			);
			if success == 0 {
				return None;
			}
			enif_map_iterator_next(self.env.raw(), &mut self.iterator);
			let key = key.assume_init();
			let value = value.assume_init();
			let key = Term::from_raw(self.env, key);
			let value = Term::from_raw(self.env, value);
			Some((key, value))
		}
	}
}
