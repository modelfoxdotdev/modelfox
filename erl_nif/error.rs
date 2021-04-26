pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
	BadArg,
	Message(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let message = match self {
			Error::BadArg => "bad argument",
			Error::Message(message) => message,
		};
		write!(f, "{}", message)
	}
}

impl std::error::Error for Error {}

impl Error {
	pub fn bad_arg() -> Error {
		Error::BadArg
	}

	pub fn message(message: impl Into<String>) -> Error {
		Error::Message(message.into())
	}
}
