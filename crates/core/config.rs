/*!
This module defines the `Config` struct, which is used to configure training a model with [`train`](crate::train::train).
*/

/// This is a configuration used for training.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
	/// Use this field to configure the loading, shuffling, and interpretation of your dataset.
	pub dataset: Dataset,
	/// Use this field to configure feature engineering.
	pub features: Features,
	// Use this field to configure training.
	pub train: Train,
}

#[derive(Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Dataset {
	/// This option controls shuffling of the dataset before splitting and training.
	pub shuffle: Shuffle,
	/// This is the fraction of the train dataset that will be set aside for choosing the best model. The default value is `0.1`.
	pub comparison_fraction: f32,
	/// If you do not provide a separate test dataset, this is the fraction of the train dataset that will be set aside after shuffling to evalute your model. The default value is `0.2`.
	pub test_fraction: f32,
	/// Use this field to specify the column types for a subset of the columns. If you do not configure a column here, its configuration will be inferred.
	pub columns: Vec<Column>,
}

/// This option controls whether the dataset should be shuffled before splitting and training.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shuffle {
	pub enable: bool,
	pub seed: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Column {
	#[serde(rename = "unknown")]
	Unknown(UnknownColumn),
	#[serde(rename = "number")]
	Number(NumberColumn),
	#[serde(rename = "enum")]
	Enum(EnumColumn),
	#[serde(rename = "text")]
	Text(TextColumn),
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownColumn {
	pub name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberColumn {
	pub name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumColumn {
	pub name: String,
	pub variants: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextColumn {
	pub name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Features {
	/// Use this field to control automatic feature engineering.
	pub auto: AutoFeatures,
	/// Use this field to include custom feature groups.
	pub include: Option<Vec<FeatureGroup>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoFeatures {
	/// Enable or disable automatic feature engineering.
	pub enable: bool,
	/// Exclude columns from automatic feature engineering.
	pub exclude_columns: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum FeatureGroup {
	#[serde(rename = "identity")]
	Identity(IdentityFeatureGroup),
	#[serde(rename = "normalized")]
	Normalized(NormalizedFeatureGroup),
	#[serde(rename = "one_hot_encoded")]
	OneHotEncoded(OneHotEncodedFeatureGroup),
	#[serde(rename = "bag_of_words")]
	BagOfWords(BagOfWordsFeatureGroup),
	#[serde(rename = "bag_of_words_cosine_similarity")]
	BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureGroup),
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityFeatureGroup {
	pub source_column_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BagOfWordsFeatureGroup {
	pub source_column_name: String,
	pub strategy: Option<BagOfWordsFeatureGroupStrategy>,
}

#[derive(Debug, serde::Deserialize)]
pub enum BagOfWordsFeatureGroupStrategy {
	#[serde(rename = "present")]
	Present,
	#[serde(rename = "count")]
	Count,
	#[serde(rename = "tfidf")]
	TfIdf,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BagOfWordsCosineSimilarityFeatureGroup {
	pub source_column_name_a: String,
	pub source_column_name_b: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Train {
	/// The `grid` specifies which models should be trained and with which hyperparameters. If you do not specify this option, a reasonable default grid will be used.
	pub grid: Option<Vec<GridItem>>,
	/// This is the metric that will be computed on the comparison dataset to choose the best model.
	pub comparison_metric: Option<ComparisonMetric>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "model")]
pub enum GridItem {
	#[serde(rename = "linear")]
	Linear(LinearGridItem),
	#[serde(rename = "tree")]
	Tree(TreeGridItem),
}

/// These are the options used for training linear models.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinearGridItem {
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This is the L2 regularization value to use when updating the model parameters.
	pub l2_regularization: Option<f32>,
	/// This is the learning rate to use when updating the model parameters.
	pub learning_rate: Option<f32>,
	/// This is the maximum number of epochs to train.
	pub max_epochs: Option<u64>,
	/// This is the number of examples to use for each batch of training.
	pub n_examples_per_batch: Option<u64>,
}

/// These are the options used for training tree models.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeGridItem {
	/// This option controls whether binned features will be laid out in row major or column major order. Each will produce the same result, but row major will be faster for datasets with more rows and fewer columns, while column major will be faster for datasets with fewer rows and more columns.
	pub binned_features_layout: Option<BinnedFeaturesLayout>,
	/// This option controls early stopping. If it is `Some`, then early stopping will be enabled. If it is `None`, then early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This option sets the L2 regularization value for continuous splits, which helps avoid overfitting.
	pub l2_regularization_for_continuous_splits: Option<f32>,
	/// This option sets the L2 regularization value for discrete splits, which helps avoid overfitting.
	pub l2_regularization_for_discrete_splits: Option<f32>,
	/// The learning rate scales the leaf values to control the effect each tree has on the output.
	pub learning_rate: Option<f32>,
	/// This is the maximum depth of a single tree. If this value is `None`, the depth will not be limited.
	pub max_depth: Option<u64>,
	/// This is the maximum number of examples to consider when determining the bin thresholds for number features.
	pub max_examples_for_computing_bin_thresholds: Option<u64>,
	/// This is the maximum number of leaf nodes in a single tree.
	pub max_leaf_nodes: Option<u64>,
	/// This is the maximum number of rounds of training that will occur. Fewer rounds may be trained if early stopping is enabled.
	pub max_rounds: Option<u64>,
	/// When computing the bin thresholds for number features, this is the maximum number of bins for valid values to create. If the number of unique values in the number feature is less than this value, the thresholds will be equal to the unique values, which can improve accuracy when number features have a small set of possible values.
	pub max_valid_bins_for_number_features: Option<u8>,
	/// A split will only be considered valid if the number of training examples sent to each of the resulting children is at least this value.
	pub min_examples_per_node: Option<u64>,
	/// A node will only be split if the best split achieves at least this minimum gain.
	pub min_gain_to_split: Option<f32>,
	/// A split will only be considered valid if the sum of hessians in each of the resulting children is at least this value.
	pub min_sum_hessians_per_node: Option<f32>,
	/// When choosing which direction each enum variant should be sent in a discrete split, the enum variants are sorted by a score computed from the sum of gradients and hessians for examples with that enum variant. This smoothing factor is added to the denominator of that score.
	pub smoothing_factor_for_discrete_bin_sorting: Option<f32>,
}

/// This enum defines whether binned features will be layed out in row major or column major order.
#[derive(Debug, serde::Deserialize)]
pub enum BinnedFeaturesLayout {
	#[serde(rename = "row_major")]
	RowMajor,
	#[serde(rename = "column_major")]
	ColumnMajor,
}

/// The parameters in this struct control how to determine whether training should stop early after each round or epoch.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EarlyStoppingOptions {
	/// This is the fraction of the dataset that is set aside to compute the early stopping metric.
	pub early_stopping_fraction: f32,
	/// If this many rounds or epochs pass by without a significant improvement in the early stopping metric over the previous round or epoch, training will be stopped early.
	pub n_rounds_without_improvement_to_stop: usize,
	/// This is the minimum descrease in the early stopping metric for a round or epoch to be considered a significant improvement over the previous round or epoch.
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ComparisonMetric {
	#[serde(rename = "mae")]
	Mae,
	#[serde(rename = "mse")]
	Mse,
	#[serde(rename = "rmse")]
	Rmse,
	#[serde(rename = "r2")]
	R2,
	#[serde(rename = "accuracy")]
	Accuracy,
	#[serde(rename = "auc")]
	Auc,
	#[serde(rename = "f1")]
	F1,
}

impl Default for Dataset {
	fn default() -> Self {
		Dataset {
			comparison_fraction: 0.1,
			test_fraction: 0.2,
			shuffle: Default::default(),
			columns: Default::default(),
		}
	}
}

impl Default for Shuffle {
	fn default() -> Self {
		Shuffle {
			enable: true,
			seed: 42,
		}
	}
}

impl Default for Features {
	fn default() -> Self {
		Features {
			auto: Default::default(),
			include: Default::default(),
		}
	}
}

impl Default for AutoFeatures {
	fn default() -> Self {
		AutoFeatures {
			enable: true,
			exclude_columns: Default::default(),
		}
	}
}

impl Default for Train {
	fn default() -> Self {
		Train {
			grid: Default::default(),
			comparison_metric: Default::default(),
		}
	}
}

impl std::fmt::Display for ComparisonMetric {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			ComparisonMetric::Mae => "Mean Absolute Error",
			ComparisonMetric::Mse => "Mean Squared Error",
			ComparisonMetric::Rmse => "Root Mean Squared Error",
			ComparisonMetric::R2 => "R2",
			ComparisonMetric::Accuracy => "Accuracy",
			ComparisonMetric::Auc => "Area Under the Receiver Operating Characteristic Curve",
			ComparisonMetric::F1 => "F1",
		};
		write!(f, "{}", s)
	}
}
