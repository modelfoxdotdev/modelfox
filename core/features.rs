/*!
This module implements Tangram's feature engineering that prepares datasets for machine learning.
*/

use crate::stats::{
	ColumnStatsOutput, EnumColumnStatsOutput, NumberColumnStatsOutput, TextColumnStatsOutput,
};

/// Choose feature groups for linear models based on the column stats.
pub fn choose_feature_groups_linear(
	column_stats: &[ColumnStatsOutput],
) -> Vec<tangram_features::FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			ColumnStatsOutput::Unknown(_) => {}
			ColumnStatsOutput::Number(column_stats) => {
				result.push(normalized_feature_group_for_column(column_stats));
			}
			ColumnStatsOutput::Enum(column_stats) => {
				result.push(one_hot_encoded_feature_group_for_column(column_stats));
			}
			ColumnStatsOutput::Text(column_stats) => {
				result.push(bag_of_words_feature_group_for_column(column_stats));
			}
		};
	}
	result
}

/// Choose feature groups for tree models based on the column stats.
pub fn choose_feature_groups_tree(
	column_stats: &[ColumnStatsOutput],
) -> Vec<tangram_features::FeatureGroup> {
	let mut result = Vec::new();
	for column_stats in column_stats.iter() {
		match column_stats {
			ColumnStatsOutput::Unknown(_) => {}
			ColumnStatsOutput::Number(_) => {
				result.push(identity_feature_group_for_column(column_stats));
			}
			ColumnStatsOutput::Enum(_) => {
				result.push(identity_feature_group_for_column(column_stats));
			}
			ColumnStatsOutput::Text(column_stats) => {
				result.push(bag_of_words_feature_group_for_column(column_stats));
			}
		};
	}
	result
}

fn identity_feature_group_for_column(
	column_stats: &ColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	tangram_features::FeatureGroup::Identity(tangram_features::IdentityFeatureGroup {
		source_column_name: column_stats.column_name().to_owned(),
	})
}

fn normalized_feature_group_for_column(
	column_stats: &NumberColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	tangram_features::FeatureGroup::Normalized(tangram_features::NormalizedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		mean: column_stats.mean,
		variance: column_stats.variance,
	})
}

fn one_hot_encoded_feature_group_for_column(
	column_stats: &EnumColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	let mut unique_values: Vec<_> = column_stats
		.histogram
		.iter()
		.map(|(value, _)| value.clone())
		.collect();
	unique_values.sort_unstable();
	tangram_features::FeatureGroup::OneHotEncoded(tangram_features::OneHotEncodedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		variants: unique_values,
	})
}

fn bag_of_words_feature_group_for_column(
	column_stats: &TextColumnStatsOutput,
) -> tangram_features::FeatureGroup {
	let strategy = tangram_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Present;
	let tokenizer = column_stats.tokenizer.clone();
	let ngrams = column_stats
		.top_ngrams
		.iter()
		.map(|(ngram, entry)| {
			(
				ngram.clone(),
				tangram_features::bag_of_words::BagOfWordsFeatureGroupNGramEntry { idf: entry.idf },
			)
		})
		.collect();
	let ngram_types = column_stats.ngram_types.to_owned();
	tangram_features::FeatureGroup::BagOfWords(tangram_features::BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		strategy,
		tokenizer,
		ngrams,
		ngram_types,
	})
}

// fn word_embedding_feature_group_for_column(
// 	column_stats: &TextColumnStatsOutput,
// ) -> tangram_features::FeatureGroup {
// 	let model = tangram_serialize::read::<tangram_model::WordEmbeddingModelReader>(bytes);
// 	let model = model.into();
// 	tangram_features::FeatureGroup::WordEmbedding(tangram_features::WordEmbeddingFeatureGroup {
// 		source_column_name: column_stats.column_name.to_owned(),
// 		tokenizer: column_stats.tokenizer.clone(),
// 		model,
// 	})
// }
