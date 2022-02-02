use super::number_stats::{NumberStats, NumberStatsOutput};
use fnv::{FnvBuildHasher, FnvHashMap, FnvHashSet};
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use num::ToPrimitive;
use tangram_text::{NGram, NGramRef, NGramType, Tokenizer};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum ProductionColumnStats {
	Unknown(UnknownProductionColumnStats),
	Number(NumberProductionColumnStats),
	Enum(EnumProductionColumnStats),
	Text(TextProductionColumnStats),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UnknownProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub row_count: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NumberProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub row_count: u64,
	pub stats: Option<NumberStats>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct EnumProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub histogram: FnvHashMap<String, u64>,
	pub invalid_count: u64,
	pub invalid_histogram: Option<FnvHashMap<String, u64>>,
	pub row_count: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TextProductionColumnStats {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub row_count: u64,
	#[serde(with = "indexmap::serde_seq")]
	pub ngrams: IndexMap<NGram, TextProductionColumnStatsNGramEntry, FnvBuildHasher>,
	pub untracked_ngram_occurence_count: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TextProductionColumnStatsNGramEntry {
	pub row_count: u64,
	pub occurrence_count: u64,
}

#[derive(Debug)]
pub enum ProductionColumnStatsOutput {
	Unknown(UnknownProductionColumnStatsOutput),
	Number(NumberProductionColumnStatsOutput),
	Enum(EnumProductionColumnStatsOutput),
	Text(TextProductionColumnStatsOutput),
}

#[derive(Debug)]
pub struct UnknownProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
}

#[derive(Debug)]
pub struct NumberProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub stats: Option<NumberStatsOutput>,
}

#[derive(Debug)]
pub struct EnumProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub histogram: Vec<(String, u64)>,
	pub invalid_count: u64,
	pub invalid_histogram: Option<Vec<(String, u64)>>,
}

#[derive(Debug)]
pub struct TextProductionColumnStatsOutput {
	pub absent_count: u64,
	pub column_name: String,
	pub invalid_count: u64,
	pub ngrams: Vec<(NGram, TextProductionColumnStatsOutputNGramEntry)>,
	pub untracked_ngram_occurence_count: u64,
}

#[derive(Debug)]
pub struct TextProductionColumnStatsOutputNGramEntry {
	pub row_count: u64,
	pub occurrence_count: u64,
}

impl ProductionColumnStats {
	pub fn new(column_stats: &tangram_model::ColumnStatsReader) -> ProductionColumnStats {
		match column_stats {
			tangram_model::ColumnStatsReader::UnknownColumn(stats) => {
				let stats = stats.read();
				let name = stats.column_name();
				ProductionColumnStats::Unknown(UnknownProductionColumnStats::new(name))
			}

			tangram_model::ColumnStatsReader::TextColumn(stats) => {
				let stats = stats.read();
				let name = stats.column_name();
				ProductionColumnStats::Text(TextProductionColumnStats::new(name))
			}

			tangram_model::ColumnStatsReader::NumberColumn(stats) => {
				let stats = stats.read();
				let name = stats.column_name();
				ProductionColumnStats::Number(NumberProductionColumnStats::new(name))
			}

			tangram_model::ColumnStatsReader::EnumColumn(stats) => {
				let stats = stats.read();
				let name = stats.column_name();

				// Read the list of this enum's variants out of the model
				let known_variants = stats
					.histogram()
					.iter()
					.map(|(value, _)| value) // Get the value, ignore the histogram count
					.collect::<Vec<_>>();

				ProductionColumnStats::Enum(EnumProductionColumnStats::new(name, &known_variants))
			}
		}
	}

	/// Get the name of the column this stats instance represents.
	pub fn column_name(&self) -> &str {
		match self {
			ProductionColumnStats::Unknown(s) => s.column_name.as_str(),
			ProductionColumnStats::Text(s) => s.column_name.as_str(),
			ProductionColumnStats::Number(s) => s.column_name.as_str(),
			ProductionColumnStats::Enum(s) => s.column_name.as_str(),
		}
	}

	/// Incorporate a data value into the statistics being tracked.
	pub fn update(&mut self, model: tangram_model::ModelReader, value: Option<&serde_json::Value>) {
		let column_name = self.column_name().to_string();
		match self {
			ProductionColumnStats::Unknown(stats) => stats.update(value),
			ProductionColumnStats::Text(stats) => {
				// To update text stats, we need to know which ngrams we're looking for, and how
				// to tokenize the input.  We'll read that configuration out of the model data.

				// Get this model's stats information for all columns
				let col_stats = match model.inner() {
					tangram_model::ModelInnerReader::Regressor(regressor) => {
						regressor.read().train_column_stats()
					}
					tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
						binary_classifier.read().train_column_stats()
					}
					tangram_model::ModelInnerReader::MulticlassClassifier(
						multiclass_classifier,
					) => multiclass_classifier.read().train_column_stats(),
				};

				// Get the stats from the model for this text column
				let text_column_stats = col_stats
					.iter()
					.find(|column| column.column_name() == column_name)
					.unwrap()
					.as_text_column()
					.unwrap();

				// Get the tokenizer configuration this column uses.
				let tokenizer: Tokenizer = text_column_stats.tokenizer().into();

				// Get which ngrams we're tracking, and the types of those ngrams.
				let tracked_ngrams: IndexSet<NGramRef> = text_column_stats
					.top_ngrams()
					.iter()
					.map(|(ngram, _)| ngram.into())
					.collect();
				let ngram_types: IndexSet<NGramType> = text_column_stats
					.ngram_types()
					.iter()
					.map(|ngram_type| match ngram_type {
						tangram_model::NGramTypeReader::Unigram(_) => NGramType::Unigram,
						tangram_model::NGramTypeReader::Bigram(_) => NGramType::Bigram,
					})
					.collect();

				// Interpret the value and update the statistics.
				stats.update(value, &tokenizer, &tracked_ngrams, &ngram_types)
			}
			ProductionColumnStats::Number(stats) => stats.update(value),
			ProductionColumnStats::Enum(stats) => stats.update(value),
		}
	}

	/// Merge this statistics information with another of the same type.
	///
	/// If [merge] is called with a different variant of [ProductionColumnStats], it's a no-op.
	pub fn merge(&mut self, other: ProductionColumnStats) {
		match self {
			ProductionColumnStats::Unknown(stats) => {
				if let ProductionColumnStats::Unknown(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Text(stats) => {
				if let ProductionColumnStats::Text(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Number(stats) => {
				if let ProductionColumnStats::Number(other) = other {
					stats.merge(other)
				}
			}
			ProductionColumnStats::Enum(stats) => {
				if let ProductionColumnStats::Enum(other) = other {
					stats.merge(other)
				}
			}
		}
	}

	pub fn finalize(self) -> ProductionColumnStatsOutput {
		match self {
			ProductionColumnStats::Unknown(stats) => {
				ProductionColumnStatsOutput::Unknown(stats.finalize())
			}
			ProductionColumnStats::Text(stats) => {
				ProductionColumnStatsOutput::Text(stats.finalize())
			}
			ProductionColumnStats::Number(stats) => {
				ProductionColumnStatsOutput::Number(stats.finalize())
			}
			ProductionColumnStats::Enum(stats) => {
				ProductionColumnStatsOutput::Enum(stats.finalize())
			}
		}
	}
}

impl UnknownProductionColumnStats {
	pub fn new(name: &str) -> UnknownProductionColumnStats {
		UnknownProductionColumnStats {
			column_name: name.to_string(),
			invalid_count: 0,
			absent_count: 0,
			row_count: 0,
		}
	}

	pub fn update(&mut self, value: Option<&serde_json::Value>) {
		self.row_count += 1;
		match value {
			None | Some(serde_json::Value::Null) => {
				self.absent_count += 1;
			}
			Some(serde_json::Value::String(value)) if value.is_empty() => {
				self.invalid_count += 1;
			}
			_ => self.invalid_count += 1,
		};
	}

	pub fn merge(&mut self, other: UnknownProductionColumnStats) {
		self.absent_count += other.absent_count;
		self.invalid_count += other.invalid_count;
		self.row_count += other.row_count;
	}

	pub fn finalize(self) -> UnknownProductionColumnStatsOutput {
		UnknownProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
		}
	}
}

impl NumberProductionColumnStats {
	pub fn new(name: &str) -> NumberProductionColumnStats {
		NumberProductionColumnStats {
			column_name: name.to_string(),
			absent_count: 0,
			invalid_count: 0,
			stats: None,
			row_count: 0,
		}
	}

	pub fn update(&mut self, value: Option<&serde_json::Value>) {
		self.row_count += 1;
		let value = match value {
			None | Some(serde_json::Value::Null) => {
				self.absent_count += 1;
				return;
			}

			Some(serde_json::Value::String(value)) => match value.parse() {
				Ok(value) => value,
				Err(_) => {
					self.invalid_count += 1;
					return;
				}
			},

			Some(serde_json::Value::Number(value)) => match value.as_f64() {
				Some(value) => value.to_f32().unwrap(),
				None => {
					self.invalid_count += 1;
					return;
				}
			},

			Some(serde_json::Value::Bool(_)) => {
				self.invalid_count += 1;
				return;
			}

			_ => {
				self.invalid_count += 1;
				return;
			}
		};
		match &mut self.stats {
			Some(stats) => stats.update(value),
			None => {
				self.stats.replace(NumberStats::new(value));
			}
		};
	}

	pub fn merge(&mut self, other: NumberProductionColumnStats) {
		match &mut self.stats {
			Some(stats) => {
				if let Some(other) = other.stats {
					stats.merge(other)
				}
			}
			None => self.stats = other.stats,
		};
		self.absent_count += other.absent_count;
		self.invalid_count += other.invalid_count;
		self.row_count += other.row_count;
	}

	pub fn finalize(self) -> NumberProductionColumnStatsOutput {
		NumberProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
			stats: self.stats.map(|s| s.finalize()),
		}
	}
}

impl EnumProductionColumnStats {
	pub fn new(name: &str, known_variants: &[&str]) -> EnumProductionColumnStats {
		// Make an empty histogram, using the values we know about.
		let histogram = known_variants
			.iter()
			.map(|value| (value.to_string(), 0))
			.collect();

		EnumProductionColumnStats {
			column_name: name.to_string(),
			invalid_count: 0,
			absent_count: 0,
			histogram,
			invalid_histogram: None,
			row_count: 0,
		}
	}

	pub fn update(&mut self, value: Option<&serde_json::Value>) {
		self.row_count += 1;
		let value = match value {
			None | Some(serde_json::Value::Null) => {
				self.absent_count += 1;
				return;
			}

			Some(serde_json::Value::Number(_)) => {
				self.invalid_count += 1;
				return;
			}
			Some(serde_json::Value::Bool(true)) => "true",
			Some(serde_json::Value::Bool(false)) => "false",
			Some(serde_json::Value::String(value)) => value,

			_ => {
				self.invalid_count += 1;
				return;
			}
		};
		match self.histogram.get_mut(value) {
			Some(count) => *count += 1,
			None => {
				self.invalid_count += 1;
				match &mut self.invalid_histogram {
					Some(histogram) => match histogram.get_mut(value) {
						Some(count) => *count += 1,
						None => {
							histogram.insert(value.into(), 1);
						}
					},
					None => {
						let mut invalid_histogram = <FnvHashMap<String, u64>>::default();
						invalid_histogram.insert(value.into(), 1);
						self.invalid_histogram = Some(invalid_histogram)
					}
				}
			}
		};
	}

	pub fn merge(&mut self, other: EnumProductionColumnStats) {
		self.invalid_count += other.invalid_count;
		self.absent_count += other.absent_count;
		for (value, count) in other.histogram.into_iter() {
			*self.histogram.entry(value).or_insert(0) += count;
		}
		self.row_count += other.row_count;
		match &mut self.invalid_histogram {
			Some(histogram) => {
				if let Some(other) = other.invalid_histogram {
					for (value, count) in other.into_iter() {
						*histogram.entry(value).or_insert(0) += count;
					}
				};
			}
			None => {
				if let Some(other) = other.invalid_histogram {
					self.invalid_histogram = Some(other);
				}
			}
		}
	}

	pub fn finalize(self) -> EnumProductionColumnStatsOutput {
		EnumProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			histogram: self.histogram.into_iter().collect(),
			invalid_count: self.invalid_count,
			invalid_histogram: self.invalid_histogram.map(|h| h.into_iter().collect()),
		}
	}
}

impl TextProductionColumnStats {
	fn new(name: &str) -> TextProductionColumnStats {
		TextProductionColumnStats {
			absent_count: 0,
			column_name: name.to_string(),
			invalid_count: 0,
			row_count: 0,
			ngrams: Default::default(),
			untracked_ngram_occurence_count: 0,
		}
	}

	pub fn update(
		&mut self,
		value: Option<&serde_json::Value>,
		tokenizer: &Tokenizer,
		tracked_ngrams: &IndexSet<NGramRef>,
		ngram_types: &IndexSet<NGramType>,
	) {
		// Pull out a string from the value, if we have it. Otherwise, include an absent or invalid value.
		self.row_count += 1;
		let value = match value {
			Some(serde_json::Value::String(value)) => value,
			None | Some(serde_json::Value::Null) => {
				self.absent_count += 1;
				return;
			}
			_ => {
				self.invalid_count += 1;
				return;
			}
		};

		let mut ngrams_for_row = FnvHashSet::default();

		let unigram_iter = ngram_types.contains(&NGramType::Unigram).then(|| {
			tokenizer
				.tokenize(value)
				.map(tangram_text::NGramRef::Unigram)
		});

		let bigram_iter = ngram_types.contains(&NGramType::Bigram).then(|| {
			tokenizer
				.tokenize(value)
				.tuple_windows()
				.map(|(token_a, token_b)| tangram_text::NGramRef::Bigram(token_a, token_b))
		});

		let ngram_iter = unigram_iter
			.into_iter()
			.flatten()
			.chain(bigram_iter.into_iter().flatten());

		for ngram in ngram_iter {
			if tracked_ngrams.contains(&ngram) {
				if let Some(entry) = self.ngrams.get_mut(&ngram) {
					entry.occurrence_count += 1;
				} else {
					self.ngrams.insert(
						ngram.to_ngram(),
						TextProductionColumnStatsNGramEntry {
							row_count: 0,
							occurrence_count: 1,
						},
					);
				}
				ngrams_for_row.insert(ngram);
			} else {
				self.untracked_ngram_occurence_count += 1;
			}
		}
		for ngram in ngrams_for_row.iter() {
			self.ngrams.get_mut(ngram).unwrap().row_count += 1;
		}
	}

	pub fn merge(&mut self, other: TextProductionColumnStats) {
		self.absent_count += other.absent_count;
		self.invalid_count += other.invalid_count;
		self.row_count += other.row_count;
		for (other_ngram, other_entry) in other.ngrams {
			if let Some(entry) = self.ngrams.get_mut(&other_ngram) {
				entry.occurrence_count += other_entry.occurrence_count;
				entry.row_count += other_entry.row_count;
			} else {
				self.ngrams.insert(other_ngram, other_entry);
			}
		}
		self.untracked_ngram_occurence_count += other.untracked_ngram_occurence_count;
	}

	pub fn finalize(self) -> TextProductionColumnStatsOutput {
		let ngrams = self
			.ngrams
			.into_iter()
			.map(|(ngram, entry)| {
				let entry = TextProductionColumnStatsOutputNGramEntry {
					row_count: entry.row_count,
					occurrence_count: entry.occurrence_count,
				};
				(ngram, entry)
			})
			.collect();
		TextProductionColumnStatsOutput {
			absent_count: self.absent_count,
			column_name: self.column_name,
			invalid_count: self.invalid_count,
			ngrams,
			untracked_ngram_occurence_count: self.untracked_ngram_occurence_count,
		}
	}
}

impl ProductionColumnStatsOutput {
	pub fn column_name(&self) -> &str {
		match self {
			ProductionColumnStatsOutput::Unknown(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Text(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Number(s) => s.column_name.as_str(),
			ProductionColumnStatsOutput::Enum(s) => s.column_name.as_str(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::Value;

	/// Ensure that updating a number statistic with `null` reports an absent value
	/// (Regression test for https://github.com/tangramdotdev/tangram/issues/85)
	#[test]
	fn null_number_is_absent() {
		let mut stats = NumberProductionColumnStats::new("number_stats");

		// Update the stats with `null`
		stats.update(Some(&Value::Null));

		// Check that the stats report an absent value correctly
		assert_eq!(
			stats.absent_count, 1,
			"The stats didn't consider 'null' to be an absent value."
		);
		assert_eq!(
			stats.invalid_count, 0,
			"The stats wrongly reported an invalid value. Should be absent instead."
		);
	}

	/// Ensure that updating an unknown statistic with `null` reports an absent value
	/// (Regression test for https://github.com/tangramdotdev/tangram/issues/85)
	#[test]
	fn null_unknown_is_absent() {
		let mut stats = UnknownProductionColumnStats::new("unknown_stat");

		// Update the stats with `null`
		stats.update(Some(&Value::Null));

		// Check that the stats report an absent value correctly
		assert_eq!(
			stats.absent_count, 1,
			"The stats didn't consider 'null' to be an absent value."
		);
		assert_eq!(
			stats.invalid_count, 0,
			"The stats wrongly reported an invalid value. Should be absent instead."
		);
	}

	/// Ensure that updating an enum statistic with `null` reports an absent value
	/// (Regression test for https://github.com/tangramdotdev/tangram/issues/85)
	#[test]
	fn null_enum_is_absent() {
		let enum_variants = &["the", "variants", "of", "the", "enum"];
		let mut stats = EnumProductionColumnStats::new("enum_stat", enum_variants);

		// Update the stats with `null`
		stats.update(Some(&Value::Null));

		// Check that the stats report an absent value correctly
		assert_eq!(
			stats.absent_count, 1,
			"The stats didn't consider 'null' to be an absent value."
		);
		assert_eq!(
			stats.invalid_count, 0,
			"The stats wrongly reported an invalid value. Should be absent instead."
		);
	}

	/// Ensure that updating a text statistic with `null` reports an absent value
	/// (Regression test for https://github.com/tangramdotdev/tangram/issues/85)
	#[test]
	fn null_text_is_absent() {
		let mut stats = TextProductionColumnStats::new("text_stat");

		// Use a dummy tokenizer/ngram info
		let tokenizer = Tokenizer::default();
		let ngram_types = &[].into_iter().collect();
		let tracked_ngrams = &[].into_iter().collect();

		// Update the stat
		stats.update(
			Some(&Value::Null),
			&tokenizer,
			&ngram_types,
			&tracked_ngrams,
		);

		// Check that the stats report an absent value correctly
		assert_eq!(
			stats.absent_count, 1,
			"The stats didn't consider 'null' to be an absent value."
		);
		assert_eq!(
			stats.invalid_count, 0,
			"The stats wrongly reported an invalid value. Should be absent instead."
		);
	}
}
