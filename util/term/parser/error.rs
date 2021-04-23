use super::table;

#[derive(Debug)]
pub enum Error {
	Io(std::io::Error),
	Utf8(std::str::Utf8Error),
	Parse(ParseError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Error::Io(err) => write!(f, "{:?}", err),
			Error::Utf8(err) => write!(f, "{}", err),
			Error::Parse(err) => write!(f, "{}", err),
		}
	}
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
	fn from(err: ParseError) -> Self {
		Error::Parse(err)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::Io(err)
	}
}

impl From<std::str::Utf8Error> for Error {
	fn from(err: std::str::Utf8Error) -> Self {
		Error::Utf8(err)
	}
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
	pub state: table::State,
	pub byte: u8,
}

impl std::fmt::Display for ParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"parse error with byte {:X} in state {:?}",
			self.byte, self.state
		)
	}
}

impl std::error::Error for ParseError {}
