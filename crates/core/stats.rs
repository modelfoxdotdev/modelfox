use fnv::{FnvBuildHasher, FnvHashSet};
use indexmap::IndexMap;
use itertools::Itertools;
use num::ToPrimitive;
use std::{cmp::Ordering, collections::BTreeMap, num::NonZeroU64};
use tangram_finite::Finite;
use tangram_progress_counter::ProgressCounter;
use tangram_table::prelude::*;
use tangram_text::{NGram, NGramType, Tokenizer};
use tangram_zip::zip;

/// This struct contains settings used to compute stats.
#[derive(Clone, Debug)]
pub struct StatsSettings {
	/// This is the maximum number of unique numeric values to store in the histogram.
	pub number_histogram_max_size: usize,
	/// This is the maximum number of ngrams to track for text columns.
	pub ngrams_max_count: usize,
	/// This setting specifies which ngram types should be computed.
	pub ngram_types: FnvHashSet<NGramType>,
}

impl Default for StatsSettings {
	fn default() -> StatsSettings {
		StatsSettings {
			number_histogram_max_size: 100,
			ngrams_max_count: 20_000,
			ngram_types: vec![NGramType::Unigram, NGramType::Bigram]
				.into_iter()
				.collect(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct TextColumnStatsSettings {
	pub tokenizer: Tokenizer,
}

/// This struct holds column stats.
#[derive(Clone, Debug)]
pub struct Stats(pub Vec<ColumnStats>);

/// This is an enum describing the different types of stats where the type matches the type of the source column.
#[derive(Clone, Debug)]
pub enum ColumnStats {
	Unknown(UnknownColumnStats),
	Number(NumberColumnStats),
	Enum(EnumColumnStats),
	Text(TextColumnStats),
}

/// This struct contains stats for unknown columns.
#[derive(Clone, Debug)]
pub struct UnknownColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	pub count: usize,
	pub invalid_count: usize,
}

/// This struct contains stats for number columns.
#[derive(Clone, Debug)]
pub struct NumberColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// The total number of values.
	pub count: usize,
	/// The total number of valid values.
	pub valid_count: usize,
	/// This is the total number of invalid values. Invalid values are values that fail to parse as finite f32.
	pub invalid_count: usize,
	/// This stores counts for each unique value.
	pub histogram: BTreeMap<Finite<f32>, usize>,
}

/// This struct contains stats for enum columns.
#[derive(Clone, Debug)]
pub struct EnumColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// This is the total number of values.
	pub count: usize,
	/// The enum variants.
	pub variants: Vec<String>,
	/// This is the total number of valid values.
	pub valid_count: usize,
	/// This is the total number of invalid values.
	pub invalid_count: usize,
	/// This is the histogram.
	pub histogram: Vec<usize>,
}

/// This struct contains stats for text columns.
#[derive(Clone, Debug)]
pub struct TextColumnStats {
	/// This is the name of the column.
	pub column_name: String,
	/// The total number of values.
	pub row_count: usize,
	/// This is the tokenizer used to split the text into tokens.
	pub tokenizer: Tokenizer,
	/// These are the types of ngrams to collect.
	pub ngram_types: FnvHashSet<NGramType>,
	/// These are stats collected on ngrams.
	pub ngrams: IndexMap<NGram, TextColumnStatsNGramEntry, FnvBuildHasher>,
}

#[derive(Clone, Debug, Default)]
pub struct TextColumnStatsNGramEntry {
	pub row_count: usize,
	pub occurrence_count: usize,
}

pub struct StatsOutput(pub Vec<ColumnStatsOutput>);

/// This enum describes the different types of column stats.
#[derive(Debug)]
pub enum ColumnStatsOutput {
	Unknown(UnknownColumnStatsOutput),
	Number(NumberColumnStatsOutput),
	Enum(EnumColumnStatsOutput),
	Text(TextColumnStatsOutput),
}

impl ColumnStatsOutput {
	/// Return the name of the source column.
	pub fn column_name(&self) -> &str {
		match self {
			ColumnStatsOutput::Unknown(value) => &value.column_name,
			ColumnStatsOutput::Number(value) => &value.column_name,
			ColumnStatsOutput::Enum(value) => &value.column_name,
			ColumnStatsOutput::Text(value) => &value.column_name,
		}
	}
}

/// This struct contains stats for unknown columns.
#[derive(Debug)]
pub struct UnknownColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: usize,
}

/// This struct contains stats for number columns.
#[derive(Debug)]
pub struct NumberColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: usize,
	/// This is a histogram mapping unique values to their counts. It is `None` if the number of unique values exceeds [`number_histogram_max_size`](StatsSettings#number_histogram_max_size).
	pub histogram: Option<Vec<(Finite<f32>, usize)>>,
	/// This is the total number of unique values.
	pub unique_count: usize,
	/// This is the max of the values in the column.
	pub max: f32,
	/// This is the mean of the values in the column.
	pub mean: f32,
	/// This is the min of the values in the column.
	pub min: f32,
	/// This is the total number of invalid values. Invalid values are values that fail to parse as floating point numbers.
	pub invalid_count: usize,
	/// This is the variance of the values in the column.
	pub variance: f32,
	/// This is the standard deviation of the values in the column. It is equal to the square root of the variance.
	pub std: f32,
	/// This is the p25, or 25th-percentile value in the column.
	pub p25: f32,
	/// This is the p50, or 50th-percentile value in the column, i.e. the median.
	pub p50: f32,
	/// This is the p75, or 75th-percentile value in the column.
	pub p75: f32,
}

/// This struct contains stats for enum columns.
#[derive(Debug)]
pub struct EnumColumnStatsOutput {
	/// This is the name of the column as it appears in the csv.
	pub column_name: String,
	/// This is the total number of examples that these stats were computed on.
	pub count: u64,
	/// This is a histogram mapping unique variants of the enum to the total count of occurrences of the variant in the dataset.
	pub histogram: Vec<(String, usize)>,
	/// This is the total number of values in the dataset that are invalid. A value is invalid if it is not one of the enum's variants.
	pub invalid_count: usize,
	/// This is the total number of unique values, excluding invalid values.
	pub unique_count: usize,
}

/// This struct contains stats for text columns.
#[derive(Debug)]
pub struct TextColumnStatsOutput {
	/// This is the name of the column.
	pub column_name: String,
	/// This is the number of rows that these stats were computed for.
	pub row_count: u64,
	/// This is the tokenizer that was used to separate values into tokens.
	pub tokenizer: Tokenizer,
	/// This is the list of ngram types.
	pub ngram_types: FnvHashSet<NGramType>,
	/// This is the number of unique ngrams encountered.
	pub ngrams_count: usize,
	/// This contains stats for up to `stats_settings.ngrams_max_count` ngrams with the highest `entry.row_count`s.
	pub top_ngrams: IndexMap<NGram, TextColumnStatsOutputTopNGramsEntry, FnvBuildHasher>,
}

/// This struct contains stats for individual ngrams.
#[derive(Clone, Debug)]
pub struct TextColumnStatsOutputTopNGramsEntry {
	/// This is the number of rows that contain at least one occurrence of this ngram.
	pub row_count: usize,
	/// This is the number of occurrences of this ngram across all rows.
	pub occurrence_count: usize,
	/// This is the inverse document frequency of this ngram. [Learn more](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
	pub idf: f32,
}

impl Stats {
	pub fn compute(
		table: &TableView,
		settings: &StatsSettings,
		handle_progress_event: &mut dyn FnMut(ProgressCounter),
	) -> Stats {
		let progress_total = table.ncols() as u64 * table.nrows() as u64;
		let progress_counter = ProgressCounter::new(progress_total);
		handle_progress_event(progress_counter.clone());
		let progress = &|progress| progress_counter.inc(progress);
		let column_stats = table
			.columns()
			.iter()
			.map(|column| ColumnStats::compute(column.view(), settings, progress))
			.collect();
		Stats(column_stats)
	}

	pub fn merge(self, other: Stats) -> Stats {
		let column_stats: Vec<ColumnStats> =
			zip!(self.0, other.0).map(|(a, b)| a.merge(b)).collect();
		Stats(column_stats)
	}

	pub fn finalize(self, settings: &StatsSettings) -> StatsOutput {
		let column_stats = self
			.0
			.into_iter()
			.map(|column_stats| column_stats.finalize(settings))
			.collect();
		StatsOutput(column_stats)
	}
}

impl ColumnStats {
	fn compute(
		column: TableColumnView,
		settings: &StatsSettings,
		progress: impl Fn(u64),
	) -> ColumnStats {
		match column {
			TableColumnView::Unknown(column) => {
				progress(column.len() as u64);
				ColumnStats::Unknown(UnknownColumnStats {
					column_name: column.name().unwrap().to_owned(),
					count: column.len(),
					invalid_count: column.len(),
				})
			}
			TableColumnView::Number(column) => ColumnStats::Number(NumberColumnStats::compute(
				column.view(),
				settings,
				progress,
			)),
			TableColumnView::Enum(column) => {
				ColumnStats::Enum(EnumColumnStats::compute(column, settings, progress))
			}
			TableColumnView::Text(column) => {
				ColumnStats::Text(TextColumnStats::compute(column, settings, progress))
			}
		}
	}

	fn merge(self, other: ColumnStats) -> ColumnStats {
		match (self, other) {
			(ColumnStats::Unknown(a), ColumnStats::Unknown(b)) => {
				ColumnStats::Unknown(UnknownColumnStats {
					column_name: a.column_name.clone(),
					count: a.count + b.count,
					invalid_count: a.invalid_count + b.invalid_count,
				})
			}
			(ColumnStats::Number(a), ColumnStats::Number(b)) => ColumnStats::Number(a.merge(b)),
			(ColumnStats::Enum(a), ColumnStats::Enum(b)) => ColumnStats::Enum(a.merge(b)),
			(ColumnStats::Text(a), ColumnStats::Text(b)) => ColumnStats::Text(a.merge(b)),
			_ => unreachable!(),
		}
	}

	fn finalize(self, settings: &StatsSettings) -> ColumnStatsOutput {
		match self {
			ColumnStats::Unknown(column_stats_output) => {
				ColumnStatsOutput::Unknown(UnknownColumnStatsOutput {
					column_name: column_stats_output.column_name,
					count: column_stats_output.count,
				})
			}
			ColumnStats::Number(column_stats_output) => {
				ColumnStatsOutput::Number(column_stats_output.finalize(settings))
			}
			ColumnStats::Enum(column_stats_output) => {
				ColumnStatsOutput::Enum(column_stats_output.finalize(settings))
			}
			ColumnStats::Text(column_stats_output) => {
				ColumnStatsOutput::Text(column_stats_output.finalize(settings))
			}
		}
	}
}

impl NumberColumnStats {
	fn compute(
		column: NumberTableColumnView,
		_settings: &StatsSettings,
		progress: impl Fn(u64),
	) -> NumberColumnStats {
		let mut stats = NumberColumnStats {
			column_name: column.name().unwrap().to_owned(),
			count: column.len(),
			histogram: BTreeMap::new(),
			invalid_count: 0,
			valid_count: 0,
		};
		for value in column.iter() {
			// If the value parses as a finite f32, add it to the histogram. Otherwise, increment the invalid count.
			if let Ok(value) = <Finite<f32>>::new(*value) {
				*stats.histogram.entry(value).or_insert(0) += 1;
				stats.valid_count += 1;
			} else {
				stats.invalid_count += 1;
			}
			progress(1);
		}
		stats
	}

	fn merge(mut self, other: NumberColumnStats) -> NumberColumnStats {
		for (value, count) in other.histogram.iter() {
			*self.histogram.entry(*value).or_insert(0) += count;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}

	fn finalize(self, settings: &StatsSettings) -> NumberColumnStatsOutput {
		let unique_values_count = self.histogram.len();
		let invalid_count = self.invalid_count;
		let histogram = if self.histogram.len() <= settings.number_histogram_max_size {
			Some(self.histogram.iter().map(|(k, v)| (*k, *v)).collect())
		} else {
			None
		};
		let min = self.histogram.iter().next().unwrap().0.get();
		let max = self.histogram.iter().next_back().unwrap().0.get();
		let total_values_count = self.valid_count.to_f32().unwrap();
		let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
		// Find the index of each quantile given the total number of values in the dataset.
		let quantile_indexes: Vec<usize> = quantiles
			.iter()
			.map(|q| ((total_values_count - 1.0) * q).trunc().to_usize().unwrap())
			.collect();
		// This is the fractiononal part of the index used to interpolate values if the index is not an integer value.
		let quantile_fracts: Vec<f32> = quantiles
			.iter()
			.map(|q| ((total_values_count - 1.0) * q).fract())
			.collect();
		let mut quantiles: Vec<Option<f32>> = vec![None; quantiles.len()];
		let mut current_count: usize = 0;
		let mut mean = 0.0;
		let mut m2 = 0.0;
		let mut iter = self.histogram.iter().peekable();
		while let Some((value, count)) = iter.next() {
			let value = value.get();
			let (new_mean, new_m2) = tangram_metrics::merge_mean_m2(
				current_count.to_u64().unwrap(),
				mean,
				m2,
				count.to_u64().unwrap(),
				value as f64,
				0.0,
			);
			mean = new_mean;
			m2 = new_m2;
			current_count += count;
			let quantiles_iter = zip!(
				quantiles.iter_mut(),
				quantile_indexes.iter(),
				quantile_fracts.iter(),
			)
			.filter(|(quantile, _, _)| quantile.is_none());
			for (quantile, index, fract) in quantiles_iter {
				match (current_count - 1).cmp(index) {
					Ordering::Equal => {
						if *fract > 0.0 {
							// Interpolate between two values.
							let next_value = iter.peek().unwrap().0.get();
							*quantile = Some(value * (1.0 - fract) + next_value * fract);
						} else {
							*quantile = Some(value);
						}
					}
					Ordering::Greater => *quantile = Some(value),
					Ordering::Less => {}
				}
			}
		}
		let quantiles: Vec<f32> = quantiles.into_iter().map(|q| q.unwrap()).collect();
		let p25 = quantiles[0];
		let p50 = quantiles[1];
		let p75 = quantiles[2];
		let mean = mean.to_f32().unwrap();
		let variance = tangram_metrics::m2_to_variance(
			m2,
			NonZeroU64::new(current_count.to_u64().unwrap()).unwrap(),
		);
		NumberColumnStatsOutput {
			column_name: self.column_name,
			count: self.count,
			histogram,
			unique_count: unique_values_count,
			max,
			mean,
			min,
			invalid_count,
			variance,
			std: variance.sqrt(),
			p25,
			p50,
			p75,
		}
	}
}

impl EnumColumnStats {
	fn compute(
		column: EnumTableColumnView,
		_settings: &StatsSettings,
		progress: impl Fn(u64),
	) -> EnumColumnStats {
		let mut histogram = vec![0; column.variants().len() + 1];
		for value in column.iter() {
			let index = value.map(|v| v.get()).unwrap_or(0);
			histogram[index] += 1;
			progress(1);
		}
		let invalid_count = histogram[0];
		EnumColumnStats {
			column_name: column.name().unwrap().to_owned(),
			count: column.len(),
			variants: column.variants().to_owned(),
			histogram,
			invalid_count,
			valid_count: 0,
		}
	}

	fn merge(mut self, other: EnumColumnStats) -> EnumColumnStats {
		for (a, b) in zip!(self.histogram.iter_mut(), other.histogram.iter()) {
			*a += b;
		}
		self.count += other.count;
		self.invalid_count += other.invalid_count;
		self.valid_count += other.valid_count;
		self
	}

	fn finalize(self, _settings: &StatsSettings) -> EnumColumnStatsOutput {
		EnumColumnStatsOutput {
			column_name: self.column_name,
			count: self.count.to_u64().unwrap(),
			invalid_count: self.invalid_count,
			unique_count: self.variants.len(),
			histogram: zip!(self.variants, self.histogram.into_iter().skip(1))
				.map(|(value, count)| (value, count))
				.collect(),
		}
	}
}

#[derive(Clone, Debug, Eq)]
struct TokenEntry(pub NGram, pub usize);

impl std::cmp::Ord for TokenEntry {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.1.cmp(&other.1)
	}
}

impl std::cmp::PartialOrd for TokenEntry {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.1.partial_cmp(&other.1)
	}
}

impl std::cmp::PartialEq for TokenEntry {
	fn eq(&self, other: &Self) -> bool {
		self.1.eq(&other.1)
	}
}

impl TextColumnStats {
	fn compute(
		column: TextTableColumnView,
		settings: &StatsSettings,
		progress: impl Fn(u64),
	) -> TextColumnStats {
		let tokenizer = Tokenizer::default();
		let mut stats = TextColumnStats {
			column_name: column.name().unwrap().to_owned(),
			row_count: column.len(),
			tokenizer,
			ngrams: IndexMap::default(),
			ngram_types: settings.ngram_types.to_owned(),
		};
		let mut ngrams_for_row = FnvHashSet::default();
		for value in column.iter() {
			ngrams_for_row.clear();
			let unigram_iter = if stats.ngram_types.contains(&NGramType::Unigram) {
				Some(
					stats
						.tokenizer
						.tokenize(value)
						.map(tangram_text::NGramRef::Unigram),
				)
			} else {
				None
			};
			let bigram_iter =
				if stats.ngram_types.contains(&NGramType::Bigram) {
					Some(
						stats.tokenizer.tokenize(value).tuple_windows().map(
							|(token_a, token_b)| tangram_text::NGramRef::Bigram(token_a, token_b),
						),
					)
				} else {
					None
				};
			let ngram_iter = unigram_iter
				.into_iter()
				.flatten()
				.chain(bigram_iter.into_iter().flatten());
			for ngram in ngram_iter {
				if let Some(entry) = stats.ngrams.get_mut(&ngram) {
					entry.occurrence_count += 1;
				} else {
					let ngram = ngram.to_ngram();
					let entry = TextColumnStatsNGramEntry {
						row_count: 0,
						occurrence_count: 1,
					};
					stats.ngrams.insert(ngram, entry);
				}
				ngrams_for_row.insert(ngram);
			}
			for ngram in ngrams_for_row.iter() {
				stats.ngrams.get_mut(ngram).unwrap().row_count += 1;
			}
			progress(1);
		}
		stats
	}

	fn merge(mut self, other: TextColumnStats) -> TextColumnStats {
		self.row_count += other.row_count;
		for (other_ngram, other_entry) in other.ngrams.into_iter() {
			if let Some(entry) = self.ngrams.get_mut(&other_ngram) {
				entry.row_count += other_entry.row_count;
				entry.occurrence_count += other_entry.occurrence_count;
			} else {
				self.ngrams.insert(other_ngram, other_entry);
			}
		}
		self
	}

	fn finalize(mut self, settings: &StatsSettings) -> TextColumnStatsOutput {
		self.ngrams
			.sort_by(|_, entry_a, _, entry_b| entry_a.row_count.cmp(&entry_b.row_count));
		let row_count = self.row_count.to_f32().unwrap();
		let ngrams_count = self.ngrams.len();
		let ngrams: IndexMap<NGram, TextColumnStatsOutputTopNGramsEntry, FnvBuildHasher> = self
			.ngrams
			.into_iter()
			.rev()
			.take(settings.ngrams_max_count)
			.map(|(ngram, entry)| {
				let entry_row_count = entry.row_count.to_f32().unwrap();
				let idf = ((1.0 + row_count) / (1.0 + entry_row_count)).ln() + 1.0;
				let entry = TextColumnStatsOutputTopNGramsEntry {
					idf,
					occurrence_count: entry.occurrence_count,
					row_count: entry.row_count,
				};
				(ngram, entry)
			})
			.collect();
		TextColumnStatsOutput {
			column_name: self.column_name,
			tokenizer: self.tokenizer,
			row_count: self.row_count.to_u64().unwrap(),
			ngram_types: settings.ngram_types.clone(),
			ngrams_count,
			top_ngrams: ngrams,
		}
	}
}
