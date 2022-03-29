/*!
This module implements ModelFox's feature engineering that prepares datasets for machine learning.
*/

use crate::{
	config,
	stats::{
		ColumnStatsOutput, EnumColumnStatsOutput, NumberColumnStatsOutput, TextColumnStatsOutput,
		TextColumnStatsOutputTopNGramsEntry,
	},
};
use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use modelfox_text::NGram;
use num::ToPrimitive;

pub fn choose_feature_groups_linear(
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<modelfox_features::FeatureGroup> {
	let mut result = Vec::new();
	let exclude_columns: std::collections::HashSet<String> = config
		.features
		.auto
		.exclude_columns
		.as_deref()
		.unwrap_or_default()
		.iter()
		.cloned()
		.collect();
	// Add the auto generated feature group types unless auto is not enabled.
	if config.features.auto.enable {
		for column_stats in column_stats.iter() {
			if !exclude_columns.contains(column_stats.column_name()) {
				if let Some(feature_group_type) = choose_feature_group_linear(column_stats, None) {
					result.push(feature_group_type)
				}
			}
		}
	}
	// Add the feature groups provided in the config.
	let feature_group_types_config = compute_feature_group_types_from_config(column_stats, config);
	result.extend(feature_group_types_config);
	result
}

pub fn choose_feature_groups_tree(
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<modelfox_features::FeatureGroup> {
	let mut result = Vec::new();
	let exclude_columns: std::collections::HashSet<String> = config
		.features
		.auto
		.exclude_columns
		.as_deref()
		.unwrap_or_default()
		.iter()
		.cloned()
		.collect();
	// Add the auto generated feature group types unless auto is not enabled.
	if config.features.auto.enable {
		for column_stats in column_stats.iter() {
			if !exclude_columns.contains(column_stats.column_name()) {
				if let Some(feature_group_type) = choose_feature_group_tree(column_stats, None) {
					result.push(feature_group_type)
				}
			}
		}
	}
	// Add the feature groups provided in the config.
	let feature_group_types_config = compute_feature_group_types_from_config(column_stats, config);
	result.extend(feature_group_types_config);
	result
}

pub fn compute_feature_group_types_from_config(
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<modelfox_features::FeatureGroup> {
	let mut result = Vec::new();
	for feature_group in config
		.features
		.include
		.as_deref()
		.unwrap_or_default()
		.iter()
	{
		match feature_group {
			config::FeatureGroup::Identity(feature_group) => {
				let column_stats = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name
					})
					.unwrap();
				result.push(identity_feature_group_for_column(column_stats))
			}
			config::FeatureGroup::Normalized(feature_group) => {
				let column_stats = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name
					})
					.unwrap();
				let column_stats = match column_stats {
					ColumnStatsOutput::Number(column_stats) => column_stats,
					_ => panic!(),
				};
				result.push(normalized_feature_group_for_column(column_stats))
			}
			config::FeatureGroup::OneHotEncoded(feature_group) => {
				let column_stats = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name
					})
					.unwrap();
				let column_stats = match column_stats {
					ColumnStatsOutput::Enum(column_stats) => column_stats,
					_ => panic!(),
				};
				result.push(one_hot_encoded_feature_group_for_column(column_stats))
			}
			config::FeatureGroup::BagOfWords(feature_group) => {
				let column_stats = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name
					})
					.unwrap();
				let column_stats = match column_stats {
					ColumnStatsOutput::Text(column_stats) => column_stats,
					_ => panic!(),
				};
				result.push(bag_of_words_feature_group_for_column(
					column_stats,
					Some(feature_group),
				))
			}
			config::FeatureGroup::BagOfWordsCosineSimilarity(feature_group) => {
				let column_stats_a = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name_a
					})
					.unwrap();
				let column_stats_a = match column_stats_a {
					ColumnStatsOutput::Text(column_stats) => column_stats,
					_ => panic!(),
				};
				let column_stats_b = column_stats
					.iter()
					.find(|column_stats| {
						column_stats.column_name() == feature_group.source_column_name_b
					})
					.unwrap();
				let column_stats_b = match column_stats_b {
					ColumnStatsOutput::Text(column_stats) => column_stats,
					_ => panic!(),
				};
				result.push(bag_of_words_cosine_similarity_feature_group_for_column(
					column_stats_a,
					column_stats_b,
					Default::default(),
				))
			}
		}
	}
	result
}

/// Choose feature group for tree models based on the column stats.
pub fn choose_feature_group_linear(
	column_stats: &ColumnStatsOutput,
	feature_group_config: Option<&config::FeatureGroup>,
) -> Option<modelfox_features::FeatureGroup> {
	match column_stats {
		ColumnStatsOutput::Unknown(_) => None,
		ColumnStatsOutput::Number(column_stats) => {
			Some(choose_feature_group_linear_number_column(column_stats))
		}
		ColumnStatsOutput::Enum(column_stats) => {
			Some(choose_feature_group_linear_enum_column(column_stats))
		}
		ColumnStatsOutput::Text(column_stats) => {
			let feature_group_config =
				feature_group_config.map(|feature_group_config| match feature_group_config {
					config::FeatureGroup::BagOfWords(feature_group_config) => feature_group_config,
					_ => unreachable!(),
				});
			Some(choose_feature_group_linear_text_column(
				column_stats,
				feature_group_config,
			))
		}
	}
}

fn choose_feature_group_linear_number_column(
	column_stats: &NumberColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	normalized_feature_group_for_column(column_stats)
}

fn choose_feature_group_linear_enum_column(
	column_stats: &EnumColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	one_hot_encoded_feature_group_for_column(column_stats)
}

fn choose_feature_group_linear_text_column(
	column_stats: &TextColumnStatsOutput,
	feature_group_config: Option<&config::BagOfWordsFeatureGroup>,
) -> modelfox_features::FeatureGroup {
	bag_of_words_feature_group_for_column(column_stats, feature_group_config)
}

/// Choose feature group for tree models based on the column stats.
fn choose_feature_group_tree(
	column_stats: &ColumnStatsOutput,
	feature_group_config: Option<&config::FeatureGroup>,
) -> Option<modelfox_features::FeatureGroup> {
	match column_stats {
		ColumnStatsOutput::Unknown(_) => None,
		ColumnStatsOutput::Number(_) => Some(choose_feature_group_tree_number_column(column_stats)),
		ColumnStatsOutput::Enum(_) => Some(choose_feature_group_tree_enum_column(column_stats)),
		ColumnStatsOutput::Text(column_stats) => {
			let feature_group_config =
				feature_group_config.map(|feature_group_config| match feature_group_config {
					config::FeatureGroup::BagOfWords(feature_group_config) => feature_group_config,
					_ => unreachable!(),
				});
			Some(choose_feature_group_tree_text_column(
				column_stats,
				feature_group_config,
			))
		}
	}
}

fn choose_feature_group_tree_number_column(
	column_stats: &ColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	identity_feature_group_for_column(column_stats)
}

fn choose_feature_group_tree_enum_column(
	column_stats: &ColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	identity_feature_group_for_column(column_stats)
}

fn choose_feature_group_tree_text_column(
	column_stats: &TextColumnStatsOutput,
	feature_group_config: Option<&config::BagOfWordsFeatureGroup>,
) -> modelfox_features::FeatureGroup {
	bag_of_words_feature_group_for_column(column_stats, feature_group_config)
}

fn identity_feature_group_for_column(
	column_stats: &ColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	modelfox_features::FeatureGroup::Identity(modelfox_features::IdentityFeatureGroup {
		source_column_name: column_stats.column_name().to_owned(),
	})
}

fn normalized_feature_group_for_column(
	column_stats: &NumberColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	modelfox_features::FeatureGroup::Normalized(modelfox_features::NormalizedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		mean: column_stats.mean,
		variance: column_stats.variance,
	})
}

fn one_hot_encoded_feature_group_for_column(
	column_stats: &EnumColumnStatsOutput,
) -> modelfox_features::FeatureGroup {
	let mut unique_values: Vec<_> = column_stats
		.histogram
		.iter()
		.map(|(value, _)| value.clone())
		.collect();
	unique_values.sort_unstable();
	modelfox_features::FeatureGroup::OneHotEncoded(modelfox_features::OneHotEncodedFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		variants: unique_values,
	})
}

fn bag_of_words_feature_group_for_column(
	column_stats: &TextColumnStatsOutput,
	feature_group: Option<&config::BagOfWordsFeatureGroup>,
) -> modelfox_features::FeatureGroup {
	let strategy = feature_group
		.as_ref()
		.and_then(|feature_group| {
			feature_group
				.strategy
				.as_ref()
				.map(|strategy| match strategy {
					config::BagOfWordsFeatureGroupStrategy::Present => {
						modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Present
					}
					config::BagOfWordsFeatureGroupStrategy::Count => {
						modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Count
					}
					config::BagOfWordsFeatureGroupStrategy::TfIdf => {
						modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::TfIdf
					}
				})
		})
		.unwrap_or(modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Present);
	let tokenizer = column_stats.tokenizer.clone();
	let ngrams = column_stats
		.top_ngrams
		.iter()
		.map(|(ngram, entry)| {
			(
				ngram.clone(),
				modelfox_features::bag_of_words::BagOfWordsFeatureGroupNGramEntry {
					idf: entry.idf,
				},
			)
		})
		.collect();
	let ngram_types = column_stats.ngram_types.to_owned();
	modelfox_features::FeatureGroup::BagOfWords(modelfox_features::BagOfWordsFeatureGroup {
		source_column_name: column_stats.column_name.to_owned(),
		strategy,
		tokenizer,
		ngrams,
		ngram_types,
	})
}

struct BagOfWordsCosineSimilarityFeatureGroupSettings {
	ngrams_max_count: usize,
}

impl Default for BagOfWordsCosineSimilarityFeatureGroupSettings {
	fn default() -> BagOfWordsCosineSimilarityFeatureGroupSettings {
		BagOfWordsCosineSimilarityFeatureGroupSettings {
			ngrams_max_count: 20000,
		}
	}
}

fn bag_of_words_cosine_similarity_feature_group_for_column(
	column_stats_a: &TextColumnStatsOutput,
	column_stats_b: &TextColumnStatsOutput,
	settings: BagOfWordsCosineSimilarityFeatureGroupSettings,
) -> modelfox_features::FeatureGroup {
	let strategy = modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::TfIdf;
	let tokenizer = column_stats_a.tokenizer.clone();
	let row_count =
		column_stats_a.row_count.to_f32().unwrap() + column_stats_b.row_count.to_f32().unwrap();
	let mut ngrams: IndexMap<NGram, TextColumnStatsOutputTopNGramsEntry, FnvBuildHasher> =
		IndexMap::default();
	for (ngram, entry) in column_stats_a
		.top_ngrams
		.iter()
		.chain(column_stats_b.top_ngrams.iter())
	{
		if let Some(entry) = ngrams.get_mut(ngram) {
			entry.row_count += entry.row_count;
			entry.occurrence_count += entry.occurrence_count;
		} else {
			ngrams.insert(ngram.to_owned(), entry.to_owned());
		}
	}
	ngrams.sort_by(|_, entry_a, _, entry_b| entry_a.row_count.cmp(&entry_b.row_count));
	let ngrams = ngrams
		.into_iter()
		.rev()
		.take(settings.ngrams_max_count)
		.map(|(ngram, entry)| {
			let entry_row_count = entry.row_count.to_f32().unwrap();
			let idf = ((1.0 + row_count) / (1.0 + entry_row_count)).ln() + 1.0;
			let entry = modelfox_features::bag_of_words::BagOfWordsFeatureGroupNGramEntry { idf };
			(ngram, entry)
		})
		.collect();
	let ngram_types = column_stats_a
		.ngram_types
		.union(&column_stats_b.ngram_types)
		.cloned()
		.collect();
	modelfox_features::FeatureGroup::BagOfWordsCosineSimilarity(
		modelfox_features::BagOfWordsCosineSimilarityFeatureGroup {
			source_column_name_a: column_stats_a.column_name.to_owned(),
			source_column_name_b: column_stats_b.column_name.to_owned(),
			strategy,
			tokenizer,
			ngrams,
			ngram_types,
		},
	)
}

// fn word_embedding_feature_group_for_column(
// 	column_stats: &TextColumnStatsOutput,
// ) -> modelfox_features::FeatureGroup {
// 	let model = buffalo::read::<modelfox_model::WordEmbeddingModelReader>(bytes);
// 	let model = model.into();
// 	modelfox_features::FeatureGroup::WordEmbedding(modelfox_features::WordEmbeddingFeatureGroup {
// 		source_column_name: column_stats.column_name.to_owned(),
// 		tokenizer: column_stats.tokenizer.clone(),
// 		model,
// 	})
// }
