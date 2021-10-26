use crate::bag_of_words::{BagOfWordsFeatureGroupNGramEntry, BagOfWordsFeatureGroupStrategy};
use fnv::{FnvBuildHasher, FnvHashSet};
use indexmap::IndexMap;
use itertools::Itertools;
use ndarray::prelude::*;
use num::ToPrimitive;
use tangram_table::{
	NumberTableColumn, TableColumn, TableColumnView, TableValue, TextTableColumnView,
};
use tangram_text::{NGram, NGramType, Tokenizer};

/**
A BagOfWordsCosineSimilarityFeatureGroup creates features for comparing two text columns using the cosine similarity of the [Bag of Words](https://en.wikipedia.org/wiki/Bag-of-words_model) representation of each text column.
*/
#[derive(Clone, Debug)]
pub struct BagOfWordsCosineSimilarityFeatureGroup {
	/// This is the name of the first text column used to compute features with this feature group.
	pub source_column_name_a: String,
	/// This is the name of the second text column used to compute features with this feature group.
	pub source_column_name_b: String,
	/// The strategy specifies how to compute feature values given the tokens in the source column.
	pub strategy: BagOfWordsFeatureGroupStrategy,
	/// This is the tokenizer used to split the text into tokens.
	pub tokenizer: Tokenizer,
	/// These are the ngram types used to create features.
	pub ngram_types: FnvHashSet<NGramType>,
	/// These are the ngrams, one for each feature in this feature group.
	pub ngrams: IndexMap<NGram, BagOfWordsFeatureGroupNGramEntry, FnvBuildHasher>,
}

impl BagOfWordsCosineSimilarityFeatureGroup {
	pub fn compute_table(
		&self,
		column_a: TableColumnView,
		column_b: TableColumnView,
		progress: &impl Fn(u64),
	) -> TableColumn {
		match (column_a, column_b) {
			(TableColumnView::Text(column_a), TableColumnView::Text(column_b)) => {
				self.compute_table_for_text_column(column_a, column_b, &|| progress(1))
			}
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column_a: TableColumnView,
		column_b: TableColumnView,
		progress: &impl Fn(),
	) {
		match (column_a, column_b) {
			(TableColumnView::Text(column_a), TableColumnView::Text(column_b)) => {
				self.compute_array_f32_for_text_column(features, column_a, column_b, progress)
			}
			_ => unimplemented!(),
		}
	}

	pub fn compute_array_value(
		&self,
		features: ArrayViewMut2<TableValue>,
		column_a: TableColumnView,
		column_b: TableColumnView,
		progress: &impl Fn(),
	) {
		match (column_a, column_b) {
			(TableColumnView::Text(column_a), TableColumnView::Text(column_b)) => {
				self.compute_array_value_for_text_column(features, column_a, column_b, progress)
			}
			_ => unimplemented!(),
		}
	}
}

impl BagOfWordsCosineSimilarityFeatureGroup {
	fn compute_table_for_text_column(
		&self,
		column_a: TextTableColumnView,
		column_b: TextTableColumnView,
		progress: &impl Fn(),
	) -> TableColumn {
		// Compute the feature values for each example.
		let mut feature_column = vec![0.0; column_a.len()];
		let mut bag_of_words_features_a = vec![0.0; self.ngrams.len()];
		let mut bag_of_words_features_b = vec![0.0; self.ngrams.len()];
		for (example_index, (value_a, value_b)) in column_a.iter().zip(column_b.iter()).enumerate()
		{
			// Reset memory of tmp feature vecs
			for v in &mut bag_of_words_features_a {
				*v = 0.0;
			}
			for v in &mut bag_of_words_features_b {
				*v = 0.0;
			}
			let feature = self.compute_bag_of_words_comparison_feature(
				value_a,
				value_b,
				bag_of_words_features_a.as_mut_slice(),
				bag_of_words_features_b.as_mut_slice(),
			);
			feature_column[example_index] = feature;
			progress();
		}
		TableColumn::Number(NumberTableColumn::new(None, feature_column))
	}

	fn compute_bag_of_words_comparison_feature(
		&self,
		value_a: &str,
		value_b: &str,
		bag_of_words_features_a: &mut [f32],
		bag_of_words_features_b: &mut [f32],
	) -> f32 {
		// Set the feature value for each token for this example.
		self.compute_bag_of_words_feature(value_a, bag_of_words_features_a);
		self.compute_bag_of_words_feature(value_b, bag_of_words_features_b);
		let mut feature = 0.0;
		for (feature_a, feature_b) in bag_of_words_features_a
			.iter()
			.zip(bag_of_words_features_b.iter())
		{
			feature += feature_a * feature_b;
		}
		feature
	}

	fn compute_bag_of_words_feature<'a>(
		&'a self,
		value: &'a str,
		bag_of_words_features: &mut [f32],
	) {
		let value_unigram_iter = if self.ngram_types.contains(&NGramType::Unigram) {
			Some(
				self.tokenizer
					.tokenize(value)
					.map(tangram_text::NGramRef::Unigram),
			)
		} else {
			None
		};
		let value_bigram_iter = if self.ngram_types.contains(&NGramType::Bigram) {
			Some(
				self.tokenizer
					.tokenize(value)
					.tuple_windows()
					.map(|(token_a, token_b)| tangram_text::NGramRef::Bigram(token_a, token_b)),
			)
		} else {
			None
		};
		let ngram_iter = value_unigram_iter
			.into_iter()
			.flatten()
			.chain(value_bigram_iter.into_iter().flatten());
		for ngram in ngram_iter {
			if let Some((ngram_index, _, ngram_entry)) = self.ngrams.get_full(&ngram) {
				match self.strategy {
					BagOfWordsFeatureGroupStrategy::Present => {
						let feature_value = 1.0;
						bag_of_words_features[ngram_index] = feature_value;
					}
					BagOfWordsFeatureGroupStrategy::Count => {
						let feature_value = 1.0;
						bag_of_words_features[ngram_index] += feature_value;
					}
					BagOfWordsFeatureGroupStrategy::TfIdf => {
						let feature_value = 1.0 * ngram_entry.idf;
						bag_of_words_features[ngram_index] += feature_value;
					}
				}
			}
		}
		let feature_values_sum_of_squares = bag_of_words_features
			.iter()
			.map(|value| value.to_f64().unwrap() * value.to_f64().unwrap())
			.sum::<f64>();
		// Normalize the feature values for this example.
		if feature_values_sum_of_squares > 0.0 {
			let norm = feature_values_sum_of_squares.sqrt();
			for feature in bag_of_words_features.iter_mut() {
				*feature /= norm.to_f32().unwrap();
			}
		}
	}

	fn compute_array_f32_for_text_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column_a: TextTableColumnView,
		column_b: TextTableColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		let mut bag_of_words_features_a = vec![0.0; self.ngrams.len()];
		let mut bag_of_words_features_b = vec![0.0; self.ngrams.len()];
		for (example_index, (value_a, value_b)) in column_a.iter().zip(column_b.iter()).enumerate()
		{
			// Reset memory of tmp feature vecs
			for v in &mut bag_of_words_features_a {
				*v = 0.0;
			}
			for v in &mut bag_of_words_features_b {
				*v = 0.0;
			}
			let feature = self.compute_bag_of_words_comparison_feature(
				value_a,
				value_b,
				bag_of_words_features_a.as_mut_slice(),
				bag_of_words_features_b.as_mut_slice(),
			);
			*features.get_mut([example_index, 0]).unwrap() = feature;
			progress();
		}
	}

	fn compute_array_value_for_text_column(
		&self,
		mut features: ArrayViewMut2<TableValue>,
		column_a: TextTableColumnView,
		column_b: TextTableColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		for feature in features.iter_mut() {
			*feature = TableValue::Number(0.0);
		}
		let mut bag_of_words_features_a = vec![0.0; self.ngrams.len()];
		let mut bag_of_words_features_b = vec![0.0; self.ngrams.len()];
		for (example_index, (value_a, value_b)) in column_a.iter().zip(column_b.iter()).enumerate()
		{
			// Reset memory of tmp feature vecs
			for v in &mut bag_of_words_features_a {
				*v = 0.0;
			}
			for v in &mut bag_of_words_features_b {
				*v = 0.0;
			}
			// Compute the feature values for each example.
			let feature = self.compute_bag_of_words_comparison_feature(
				value_a,
				value_b,
				bag_of_words_features_a.as_mut_slice(),
				bag_of_words_features_b.as_mut_slice(),
			);
			*features
				.get_mut([example_index, 0])
				.unwrap()
				.as_number_mut()
				.unwrap() = feature;
			progress();
		}
	}
}

#[cfg(test)]
mod test {
	use crate::bag_of_words::{BagOfWordsFeatureGroupNGramEntry, BagOfWordsFeatureGroupStrategy};
	use crate::bag_of_words_cosine_similarity::*;
	use tangram_text::{NGram, NGramType, Tokenizer};

	#[test]
	fn test_compute_bag_of_words_feature() {
		let feature_group = BagOfWordsCosineSimilarityFeatureGroup {
			source_column_name_a: "column_a".to_owned(),
			source_column_name_b: "column_b".to_owned(),
			strategy: BagOfWordsFeatureGroupStrategy::Present,
			tokenizer: Tokenizer::default(),
			ngram_types: vec![NGramType::Unigram].into_iter().collect(),
			ngrams: vec![
				(
					NGram::Unigram("test".to_owned()),
					BagOfWordsFeatureGroupNGramEntry { idf: 1.0 },
				),
				(
					NGram::Unigram("hello".to_owned()),
					BagOfWordsFeatureGroupNGramEntry { idf: 0.3 },
				),
			]
			.into_iter()
			.collect(),
		};
		let mut bag_of_words_features = vec![0.0; feature_group.ngrams.len()];
		feature_group.compute_bag_of_words_feature(
			&"hello".to_owned(),
			bag_of_words_features.as_mut_slice(),
		);
		assert!((bag_of_words_features[0] - 0.0).abs() < f32::EPSILON);
		assert!((bag_of_words_features[1] - 1.0).abs() < f32::EPSILON);

		let mut bag_of_words_features = vec![0.0; feature_group.ngrams.len()];
		feature_group.compute_bag_of_words_feature(
			&"hello test".to_owned(),
			bag_of_words_features.as_mut_slice(),
		);
		assert!((bag_of_words_features[0] - 1.0 / 2.0f32.sqrt()).abs() < f32::EPSILON);
		assert!((bag_of_words_features[1] - 1.0 / 2.0f32.sqrt()).abs() < f32::EPSILON);
	}

	#[test]
	fn test_compute_bag_of_words_comparison_feature() {
		let feature_group = BagOfWordsCosineSimilarityFeatureGroup {
			source_column_name_a: "column_a".to_owned(),
			source_column_name_b: "column_b".to_owned(),
			strategy: BagOfWordsFeatureGroupStrategy::Present,
			tokenizer: Tokenizer::default(),
			ngram_types: vec![NGramType::Unigram].into_iter().collect(),
			ngrams: vec![
				(
					NGram::Unigram("test".to_owned()),
					BagOfWordsFeatureGroupNGramEntry { idf: 1.0 },
				),
				(
					NGram::Unigram("ben".to_owned()),
					BagOfWordsFeatureGroupNGramEntry { idf: 0.3 },
				),
				(
					NGram::Unigram("bitdiddle".to_owned()),
					BagOfWordsFeatureGroupNGramEntry { idf: 0.3 },
				),
			]
			.into_iter()
			.collect(),
		};
		let mut bag_of_words_features_a = vec![0.0; feature_group.ngrams.len()];
		let mut bag_of_words_features_b = vec![0.0; feature_group.ngrams.len()];
		let feature = feature_group.compute_bag_of_words_comparison_feature(
			&"Ben Bitdiddle".to_owned(),
			&"Little Ben Bitdiddle".to_owned(),
			bag_of_words_features_a.as_mut_slice(),
			bag_of_words_features_b.as_mut_slice(),
		);
		let right = 1.0;
		assert!(feature - right < std::f32::EPSILON);
	}
}
