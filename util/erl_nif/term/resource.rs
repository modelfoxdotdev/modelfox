use crate::{
	sys::{
		enif_alloc_resource, enif_get_resource, enif_keep_resource, enif_make_resource,
		enif_open_resource_type, enif_release_resource, ErlNifEnv, ErlNifResourceFlags,
		ErlNifResourceType,
	},
	Env, Error, Result, Term,
};
use std::{marker::PhantomData, mem::MaybeUninit, os::raw::c_void};

pub struct ResourceType<T> {
	pointer: *mut ErlNifResourceType,
	marker: PhantomData<T>,
}

unsafe impl<T> Sync for ResourceType<T> {}
unsafe impl<T> Send for ResourceType<T> {}

impl<T> Clone for ResourceType<T> {
	fn clone(&self) -> Self {
		ResourceType {
			pointer: self.pointer,
			marker: PhantomData,
		}
	}
}

impl<T> Copy for ResourceType<T> {}

impl<T> ResourceType<T> {
	pub(crate) fn from_raw(pointer: *mut ErlNifResourceType) -> ResourceType<T> {
		ResourceType {
			pointer,
			marker: PhantomData,
		}
	}

	pub fn new(env: Env, name: &str) -> Result<ResourceType<T>> {
		unsafe extern "C" fn destructor<T>(_env: *mut ErlNifEnv, value: *mut c_void) {
			drop(std::ptr::read::<T>(value as *mut T));
		}
		let pointer = unsafe {
			let name = std::ffi::CString::new(name).unwrap();
			let mut tried = MaybeUninit::uninit();
			let pointer = enif_open_resource_type(
				env.raw(),
				std::ptr::null(),
				name.as_ptr(),
				Some(destructor::<T>),
				ErlNifResourceFlags::ERL_NIF_RT_CREATE,
				tried.as_mut_ptr(),
			);
			if pointer == std::ptr::null_mut() {
				return Err(Error::message("failed to create resource type"));
			}
			let tried = tried.assume_init();
			if tried != ErlNifResourceFlags::ERL_NIF_RT_CREATE {
				return Err(Error::message("failed to create resource type"));
			}
			pointer
		};
		let resource_type = ResourceType::from_raw(pointer);
		Ok(resource_type)
	}

	pub fn raw(&self) -> *mut ErlNifResourceType {
		self.pointer
	}
}

pub struct Resource<T> {
	pointer: *mut T,
	resource_type: ResourceType<T>,
}

impl<T> Clone for Resource<T> {
	fn clone(&self) -> Self {
		unsafe { enif_keep_resource(self.pointer as *mut c_void) }
		Resource {
			pointer: self.pointer,
			resource_type: self.resource_type,
		}
	}
}

impl<T> Drop for Resource<T> {
	fn drop(&mut self) {
		unsafe { enif_release_resource(self.pointer as *mut c_void) }
	}
}

impl<T> Resource<T> {
	pub fn new(resource_type: ResourceType<T>, value: T) -> Resource<T> {
		let pointer =
			unsafe { enif_alloc_resource(resource_type.raw(), std::mem::size_of::<T>()) as *mut T };
		unsafe { std::ptr::write(pointer, value) };
		Resource {
			pointer,
			resource_type,
		}
	}

	pub fn get(&self) -> &T {
		unsafe { &*self.pointer }
	}

	pub fn get_mut(&mut self) -> &mut T {
		unsafe { &mut *self.pointer }
	}
}

pub struct ResourceTerm<'a, T> {
	term: Term<'a>,
	resource: Resource<T>,
}

impl<'a, T> ResourceTerm<'a, T> {
	pub(crate) fn from_term(
		term: Term<'a>,
		resource_type: ResourceType<T>,
	) -> Result<ResourceTerm<'a, T>> {
		let pointer = unsafe {
			let mut value = MaybeUninit::uninit();
			let success = enif_get_resource(
				term.env().raw(),
				term.raw(),
				resource_type.raw(),
				value.as_mut_ptr(),
			);
			if success == 0 {
				return Err(Error::message("unable to get resource"));
			}
			value.assume_init() as *mut T
		};
		unsafe { enif_keep_resource(pointer as *mut c_void) }
		let resource = Resource {
			pointer,
			resource_type,
		};
		Ok(ResourceTerm { term, resource })
	}

	pub fn term(&self) -> Term<'a> {
		self.term
	}

	pub fn env(&self) -> Env<'a> {
		self.term.env()
	}

	pub fn new(env: Env<'a>, resource: Resource<T>) -> Result<ResourceTerm<'a, T>> {
		let term = unsafe { enif_make_resource(env.raw(), resource.pointer as *mut c_void) };
		let term = Term::from_raw(env, term);
		Ok(ResourceTerm { term, resource })
	}

	pub fn get(&self) -> Result<&T> {
		Ok(self.resource.get())
	}
}
