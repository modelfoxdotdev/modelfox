use crate::sys::*;
use std::ffi::CStr;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
	NodeApi(napi_status, String),
	Message(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let message = match self {
			Error::NodeApi(_, message) => message,
			Error::Message(message) => message,
		};
		write!(f, "{}", message)
	}
}

impl std::error::Error for Error {}

impl Error {
	pub fn node_api(status: napi_status, message: impl Into<String>) -> Error {
		Error::NodeApi(status, message.into())
	}

	pub fn message(message: impl Into<String>) -> Error {
		Error::Message(message.into())
	}

	/// Create a `NodeAPIError` value from the most recent Node-API call.
	pub fn from_last_node_api_error(env: napi_env, status: napi_status) -> Error {
		unsafe {
			let mut error_info = std::mem::MaybeUninit::uninit();
			let last_error_status = napi_get_last_error_info(env, error_info.as_mut_ptr());
			if last_error_status != napi_status::napi_ok {
				napi_fatal_error(std::ptr::null(), 0, std::ptr::null(), 0);
				unreachable!()
			}
			let error_info = error_info.assume_init();
			let message = (*error_info).error_message;
			let message = if !message.is_null() {
				match CStr::from_ptr(message).to_str() {
					Ok(message) => message,
					Err(_) => {
						napi_fatal_error(std::ptr::null(), 0, std::ptr::null(), 0);
						unreachable!()
					}
				}
			} else {
				"empty error message"
			};
			let message = message.to_owned();
			Error::NodeApi(status, message)
		}
	}
}
