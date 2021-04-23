use crate::{
	bag_of_words::BagOfWordsFeatureGroup, identity::IdentityFeatureGroup,
	normalized::NormalizedFeatureGroup, one_hot_encoded::OneHotEncodedFeatureGroup,
	word_embedding::WordEmbeddingFeatureGroup, FeatureGroup,
};
use ndarray::prelude::*;
use tangram_table::prelude::*;

/// Compute features as an `Array` of `f32`s.
pub fn compute_features_array_f32(
	table: &TableView,
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(),
) -> Array2<f32> {
	let n_features = feature_groups
		.iter()
		.map(|feature_group| feature_group.n_features())
		.sum::<usize>();
	let mut features = Array::zeros((table.nrows(), n_features));
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		compute_features_array_f32_for_feature_group(table, feature_group, features, progress);
		feature_index += n_features_in_group;
	}
	features
}

fn compute_features_array_f32_for_feature_group(
	table: &TableView,
	feature_group: &FeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	match &feature_group {
		FeatureGroup::Identity(feature_group) => {
			compute_features_array_f32_for_identity_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::Normalized(feature_group) => {
			compute_features_array_f32_for_normalized_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::OneHotEncoded(feature_group) => {
			compute_features_array_f32_for_one_hot_encoded_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::BagOfWords(feature_group) => {
			compute_features_array_f32_for_bag_of_words_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::WordEmbedding(feature_group) => {
			compute_features_array_f32_for_word_embedding_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
	}
}

fn compute_features_array_f32_for_identity_feature_group(
	table: &TableView,
	feature_group: &IdentityFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

fn compute_features_array_f32_for_normalized_feature_group(
	table: &TableView,
	feature_group: &NormalizedFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress)
}

fn compute_features_array_f32_for_one_hot_encoded_feature_group(
	table: &TableView,
	feature_group: &OneHotEncodedFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

fn compute_features_array_f32_for_bag_of_words_feature_group(
	table: &TableView,
	feature_group: &BagOfWordsFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

fn compute_features_array_f32_for_word_embedding_feature_group(
	table: &TableView,
	feature_group: &WordEmbeddingFeatureGroup,
	features: ArrayViewMut2<f32>,
	progress: &impl Fn(),
) {
	// Get the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name() == Some(&feature_group.source_column_name))
		.unwrap();
	feature_group.compute_array_f32(features, source_column.view(), progress);
}

/// Compute features as a `Table`.
pub fn compute_features_table(
	table: &TableView,
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(u64),
) -> Table {
	let mut features = Table::new(Vec::new(), Vec::new());
	for feature_group in feature_groups.iter() {
		compute_features_table_for_feature_group(table, feature_group, &mut features, progress)
	}
	features
}

fn compute_features_table_for_feature_group(
	table: &TableView,
	feature_group: &FeatureGroup,
	features: &mut Table,
	progress: &impl Fn(u64),
) {
	match &feature_group {
		FeatureGroup::Identity(feature_group) => compute_features_table_for_identity_feature_group(
			table,
			feature_group,
			features,
			progress,
		),
		FeatureGroup::Normalized(feature_group) => {
			compute_features_table_for_normalized_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::OneHotEncoded(_) => unimplemented!(),
		FeatureGroup::BagOfWords(feature_group) => {
			compute_features_table_for_bag_of_words_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::WordEmbedding(feature_group) => {
			compute_features_table_for_word_embedding_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
	};
}

fn compute_features_table_for_identity_feature_group(
	table: &TableView,
	feature_group: &IdentityFeatureGroup,
	features: &mut Table,
	progress: &impl Fn(u64),
) {
	let column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let feature_column = feature_group.compute_table(column.view(), progress);
	features.columns_mut().push(feature_column);
}

fn compute_features_table_for_normalized_feature_group(
	table: &TableView,
	feature_group: &NormalizedFeatureGroup,
	features: &mut Table,
	progress: &impl Fn(u64),
) {
	let column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let feature_column = feature_group.compute_table(column.view(), progress);
	features.columns_mut().push(feature_column);
}

fn compute_features_table_for_bag_of_words_feature_group(
	table: &TableView,
	feature_group: &BagOfWordsFeatureGroup,
	features: &mut Table,
	progress: &impl Fn(u64),
) {
	// Get the data for the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let columns = feature_group.compute_table(source_column.view(), progress);
	for column in columns {
		features.columns_mut().push(column);
	}
}

fn compute_features_table_for_word_embedding_feature_group(
	table: &TableView,
	feature_group: &WordEmbeddingFeatureGroup,
	features: &mut Table,
	progress: &impl Fn(u64),
) {
	// Get the data for the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	let columns = feature_group.compute_table(source_column.view(), progress);
	for column in columns {
		features.columns_mut().push(column);
	}
}

pub fn compute_features_array_value<'a>(
	table: &TableView<'a>,
	feature_groups: &[FeatureGroup],
	progress: &impl Fn(),
) -> Array2<TableValue<'a>> {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let mut features = Array::from_elem((table.nrows(), n_features), TableValue::Unknown);
	let mut feature_index = 0;
	for feature_group in feature_groups.iter() {
		let n_features_in_group = feature_group.n_features();
		let slice = s![.., feature_index..feature_index + n_features_in_group];
		let features = features.slice_mut(slice);
		compute_features_array_value_for_feature_group(table, feature_group, features, progress);
		feature_index += n_features_in_group;
	}
	features
}

fn compute_features_array_value_for_feature_group(
	table: &TableView,
	feature_group: &FeatureGroup,
	features: ArrayViewMut2<tangram_table::TableValue>,
	progress: &impl Fn(),
) {
	match &feature_group {
		FeatureGroup::Identity(feature_group) => {
			compute_features_array_value_for_identity_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::Normalized(feature_group) => {
			compute_features_array_value_for_normalized_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::OneHotEncoded(_) => unimplemented!(),
		FeatureGroup::BagOfWords(feature_group) => {
			compute_features_array_value_for_bag_of_words_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
		FeatureGroup::WordEmbedding(feature_group) => {
			compute_features_array_value_for_word_embedding_feature_group(
				table,
				feature_group,
				features,
				progress,
			)
		}
	}
}

fn compute_features_array_value_for_identity_feature_group(
	table: &TableView,
	feature_group: &IdentityFeatureGroup,
	features: ArrayViewMut2<tangram_table::TableValue>,
	progress: &impl Fn(),
) {
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_array_value(features, source_column.view(), progress);
}

fn compute_features_array_value_for_normalized_feature_group(
	table: &TableView,
	feature_group: &NormalizedFeatureGroup,
	features: ArrayViewMut2<tangram_table::TableValue>,
	progress: &impl Fn(),
) {
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_array_value(features, source_column.view(), progress);
}

fn compute_features_array_value_for_bag_of_words_feature_group(
	table: &TableView,
	feature_group: &BagOfWordsFeatureGroup,
	features: ArrayViewMut2<tangram_table::TableValue>,
	progress: &impl Fn(),
) {
	// Get the data for the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_array_value(features, source_column.view(), progress);
}

fn compute_features_array_value_for_word_embedding_feature_group(
	table: &TableView,
	feature_group: &WordEmbeddingFeatureGroup,
	features: ArrayViewMut2<tangram_table::TableValue>,
	progress: &impl Fn(),
) {
	// Get the data for the source column.
	let source_column = table
		.columns()
		.iter()
		.find(|column| column.name().unwrap() == feature_group.source_column_name)
		.unwrap();
	feature_group.compute_array_value(features, source_column.view(), progress);
}
