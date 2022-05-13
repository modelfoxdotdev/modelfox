/*!
This crate defines the structure of `.modelfox` files using the `buffalo` crate.
*/

pub use self::{
	binary_classifier::*, features::*, grid::*, model_train_options::*, multiclass_classifier::*,
	regressor::*, stats::*,
};
use anyhow::{bail, Result};
use fnv::FnvHashMap;
use num::ToPrimitive;
use std::{io::prelude::*, path::Path};

mod binary_classifier;
mod features;
mod grid;
mod model_train_options;
mod multiclass_classifier;
mod regressor;
mod stats;

/// A .modelfox file is prefixed with this magic number followed by a 4-byte little endian revision number.
const MAGIC_NUMBER: &[u8] = b"tangram\0";
/// This is the revision number that this version of modelfox_model writes.
const CURRENT_REVISION: u32 = 0;
/// This is the oldest revision number that this version of modelfox_model can read.
const MIN_SUPPORTED_REVISION: u32 = 0;

pub fn from_bytes(bytes: &[u8]) -> Result<ModelReader> {
	// Verify the magic number.
	let magic_number = &bytes[0..MAGIC_NUMBER.len()];
	if magic_number != MAGIC_NUMBER {
		bail!("This model did not start with the modelfox magic number. Are you sure it is a .modelfox file?");
	}
	let bytes = &bytes[MAGIC_NUMBER.len()..];
	let revision = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
	#[allow(clippy::absurd_extreme_comparisons)]
	if revision > CURRENT_REVISION {
		bail!("This model has a revision number of {}, which is greater than the revision number of {} used by this version of modelfox. Your model is from the future! Please update to the latest version of modelfox to use it.", revision, CURRENT_REVISION);
	}
	#[allow(clippy::absurd_extreme_comparisons)]
	if revision < MIN_SUPPORTED_REVISION {
		bail!("This model has a revision number of {}, which is lower than the minumum supported revision number of {} for this version of modelfox. Please downgrade to an earlier version of modelfox to use it.", revision, MIN_SUPPORTED_REVISION);
	}
	let bytes = &bytes[4..];
	let model = buffalo::read::<ModelReader>(bytes);
	Ok(model)
}

pub fn to_path(path: &Path, bytes: &[u8]) -> Result<()> {
	// Create the file.
	let mut file = std::fs::File::create(path)?;
	// Write the magic number.
	file.write_all(MAGIC_NUMBER)?;
	// Write the revision number.
	file.write_all(&CURRENT_REVISION.to_le_bytes())?;
	// Write the bytes.
	file.write_all(bytes)?;
	Ok(())
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Model {
	#[buffalo(id = 0, required)]
	pub id: String,
	#[buffalo(id = 1, required)]
	pub version: String,
	#[buffalo(id = 2, required)]
	pub date: String,
	#[buffalo(id = 3, required)]
	pub inner: ModelInner,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
#[allow(clippy::large_enum_variant)]
pub enum ModelInner {
	#[buffalo(id = 0)]
	Regressor(Regressor),
	#[buffalo(id = 1)]
	BinaryClassifier(BinaryClassifier),
	#[buffalo(id = 2)]
	MulticlassClassifier(MulticlassClassifier),
}

impl<'a> ColumnStatsReader<'a> {
	pub fn column_name(&self) -> &str {
		match &self {
			ColumnStatsReader::UnknownColumn(c) => c.read().column_name(),
			ColumnStatsReader::NumberColumn(c) => c.read().column_name(),
			ColumnStatsReader::EnumColumn(c) => c.read().column_name(),
			ColumnStatsReader::TextColumn(c) => c.read().column_name(),
		}
	}
}

impl<'a> std::fmt::Display for NGramReader<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NGramReader::Unigram(token) => {
				let token = token.read();
				write!(f, "{}", token)
			}
			NGramReader::Bigram(token) => {
				let token = token.read();
				write!(f, "{} {}", token.0, token.1)
			}
		}
	}
}

impl<'a> From<TokenizerReader<'a>> for modelfox_text::Tokenizer {
	fn from(value: TokenizerReader<'a>) -> Self {
		modelfox_text::Tokenizer {
			lowercase: value.lowercase(),
			alphanumeric: value.alphanumeric(),
		}
	}
}

impl<'a> From<NGramReader<'a>> for modelfox_text::NGramRef<'a> {
	fn from(value: NGramReader<'a>) -> Self {
		match value {
			NGramReader::Unigram(token) => {
				let token = token.read();
				modelfox_text::NGramRef::Unigram((*token).into())
			}
			NGramReader::Bigram(bigram) => {
				let bigram = bigram.read();
				modelfox_text::NGramRef::Bigram(bigram.0.into(), bigram.1.into())
			}
		}
	}
}

impl<'a> From<WordEmbeddingModelReader<'a>> for modelfox_text::WordEmbeddingModel {
	fn from(value: WordEmbeddingModelReader<'a>) -> Self {
		let size = value.size().to_usize().unwrap();
		let words = value
			.words()
			.iter()
			.map(|(word, index)| (word.to_owned(), index.to_usize().unwrap()))
			.collect::<FnvHashMap<_, _>>();
		let values = value.values().iter().collect();
		modelfox_text::WordEmbeddingModel {
			size,
			words,
			values,
		}
	}
}
