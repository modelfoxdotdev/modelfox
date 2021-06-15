/*!
This crate provides bindings to the [Node-API](https://nodejs.org/docs/latest/api/n-api.html), making it possible to write Node native addons in Rust. The `node_api_sys` crate provides direct bindings to the Node-API C API. The `node_api` crate provides high level bindings that are designed to be the Rust equivalent of the [node-addon-api](https://github.com/nodejs/node-addon-api) for C++.
*/

use std::os::raw::c_char;

mod convert;
mod env;
mod error;
#[cfg(feature = "serde_1")]
mod serde;
mod value;

pub use self::{
	convert::{FromNodeAPI, ToNodeAPI},
	env::Env,
	error::{Error, Result},
	value::{
		array::ArrayIterator, Array, ArrayBuffer, BigInt, Boolean, Buffer, DataView, Date,
		External, Function, Null, Number, Object, String, Symbol, TypedArray, Undefined, Value,
	},
};
pub use node_api_macro::{function, init};
pub use node_api_sys as sys;

pub fn fatal_error(location: Option<&str>, message: Option<&str>) {
	unsafe {
		sys::napi_fatal_error(
			location
				.map(|location| location.as_ptr())
				.unwrap_or_else(std::ptr::null) as *const c_char,
			location.map(|location| location.len()).unwrap_or(0),
			message
				.map(|message| message.as_ptr())
				.unwrap_or_else(std::ptr::null) as *const c_char,
			message.map(|location| location.len()).unwrap_or(0),
		)
	}
}
