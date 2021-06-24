use std::borrow::Cow;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tokenizer {
	pub lowercase: bool,
	pub alphanumeric: bool,
}

impl Default for Tokenizer {
	fn default() -> Self {
		Tokenizer {
			lowercase: true,
			alphanumeric: true,
		}
	}
}

impl Tokenizer {
	pub fn tokenize<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Cow<'a, str>> {
		TokenizerIterator {
			cursor: StrCursor::new(text),
			tokenizer: self,
		}
	}
}

#[derive(Clone, Debug)]
pub struct TokenizerIterator<'a> {
	cursor: StrCursor<'a>,
	tokenizer: &'a Tokenizer,
}

impl<'a> Iterator for TokenizerIterator<'a> {
	type Item = Cow<'a, str>;
	fn next(&mut self) -> Option<Self::Item> {
		// Pass over any leading whitespace.
		while self.cursor.peek()?.is_whitespace() {
			self.cursor.next();
		}
		// Mark the start of the token.
		let token_start_index = self.cursor.index();
		let first_char = self.cursor.next().unwrap();
		let mut token_contains_uppercase = first_char.is_uppercase();
		// If the first char is alphanumeric, include any subsequent alphanumeric chars.
		if first_char.is_alphanumeric() {
			while self
				.cursor
				.peek()
				.map(|c| c.is_alphanumeric())
				.unwrap_or(false)
			{
				let c = self.cursor.next().unwrap();
				token_contains_uppercase |= c.is_uppercase();
			}
		}
		let token = &self.cursor.string[token_start_index..self.cursor.index()];
		let token = if self.tokenizer.lowercase && token_contains_uppercase {
			Cow::Owned(token.to_lowercase())
		} else {
			Cow::Borrowed(token)
		};
		Some(token)
	}
}

#[derive(Clone, Debug)]
struct StrCursor<'a> {
	string: &'a str,
	index: usize,
}

impl<'a> StrCursor<'a> {
	pub fn new(string: &'a str) -> StrCursor<'a> {
		StrCursor { string, index: 0 }
	}

	pub fn index(&self) -> usize {
		self.index
	}

	pub fn peek(&self) -> Option<char> {
		self.string[self.index..].chars().next()
	}

	pub fn next(&mut self) -> Option<char> {
		let c = self.string[self.index..].chars().next()?;
		self.index += c.len_utf8();
		Some(c)
	}
}

#[test]
fn test_tokenizer() {
	fn test(tokenizer: Tokenizer, left: &str, right: Vec<&str>) {
		assert!(tokenizer.tokenize(left).eq(right));
	}
	test(
		Tokenizer {
			lowercase: false,
			..Default::default()
		},
		"",
		vec![],
	);
	test(
		Tokenizer {
			lowercase: false,
			..Default::default()
		},
		"   ",
		vec![],
	);
	test(
		Tokenizer {
			lowercase: false,
			..Default::default()
		},
		" &*! ",
		vec!["&", "*", "!"],
	);
	test(
		Tokenizer {
			..Default::default()
		},
		"Founder/CEO",
		vec!["founder", "/", "ceo"],
	);
	test(
		Tokenizer {
			..Default::default()
		},
		"iOS Developer",
		vec!["ios", "developer"],
	);
}
