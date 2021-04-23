use std::{borrow::Cow, fmt::Display, hash::Hash};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum NGram {
	Unigram(String),
	Bigram(String, String),
}

impl PartialEq for NGram {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(NGram::Unigram(self_token), NGram::Unigram(other_token)) => self_token == other_token,
			(
				NGram::Bigram(self_token_a, self_token_b),
				NGram::Bigram(other_token_a, other_token_b),
			) => self_token_a == other_token_a && self_token_b == other_token_b,
			_ => false,
		}
	}
}

impl Hash for NGram {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			NGram::Unigram(token) => {
				0usize.hash(state);
				token.hash(state)
			}
			NGram::Bigram(token_a, token_b) => {
				1usize.hash(state);
				token_a.hash(state);
				token_b.hash(state);
			}
		}
	}
}

impl Display for NGram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NGram::Unigram(token) => {
				write!(f, "{}", token)
			}
			NGram::Bigram(token_a, token_b) => {
				write!(f, "{} {}", token_a, token_b)
			}
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialOrd, Ord)]
pub enum NGramRef<'a> {
	Unigram(Cow<'a, str>),
	Bigram(Cow<'a, str>, Cow<'a, str>),
}

impl<'a> PartialEq for NGramRef<'a> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(NGramRef::Unigram(self_token), NGramRef::Unigram(other_token)) => {
				self_token == other_token
			}
			(
				NGramRef::Bigram(self_token_a, self_token_b),
				NGramRef::Bigram(other_token_a, other_token_b),
			) => self_token_a == other_token_a && self_token_b == other_token_b,
			_ => false,
		}
	}
}

impl<'a> Hash for NGramRef<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			NGramRef::Unigram(token) => {
				0usize.hash(state);
				token.hash(state)
			}
			NGramRef::Bigram(token_a, token_b) => {
				1usize.hash(state);
				token_a.hash(state);
				token_b.hash(state);
			}
		}
	}
}

impl<'a> indexmap::Equivalent<NGram> for NGramRef<'a> {
	fn equivalent(&self, key: &NGram) -> bool {
		match (self, key) {
			(NGramRef::Unigram(unigram_ref), NGram::Unigram(unigram)) => unigram_ref == unigram,
			(NGramRef::Bigram(bigram_a_ref, bigram_b_ref), NGram::Bigram(bigram_a, bigram_b)) => {
				bigram_a_ref == bigram_a && bigram_b_ref == bigram_b
			}
			_ => false,
		}
	}
}

impl<'a> NGramRef<'a> {
	pub fn to_ngram(&self) -> NGram {
		match self {
			NGramRef::Unigram(token) => NGram::Unigram(token.as_ref().to_owned()),
			NGramRef::Bigram(token_a, token_b) => {
				NGram::Bigram(token_a.as_ref().to_owned(), token_b.as_ref().to_owned())
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NGramType {
	Unigram,
	Bigram,
}
