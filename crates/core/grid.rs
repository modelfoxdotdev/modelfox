use crate::{
	config,
	features::{choose_feature_groups_linear, choose_feature_groups_tree},
	stats::ColumnStatsOutput,
};
use itertools::iproduct;

/// A `GridItem` is a description of a single entry in a hyperparameter grid. It specifies what feature engineering to perform on the training data, which model to train, and which hyperparameters to use.
#[derive(Clone, Debug)]
pub enum GridItem {
	LinearRegressor {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeRegressor {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
	LinearBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeBinaryClassifier {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
	LinearMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: LinearModelTrainOptions,
	},
	TreeMulticlassClassifier {
		target_column_index: usize,
		feature_groups: Vec<tangram_features::FeatureGroup>,
		options: TreeModelTrainOptions,
	},
}

#[derive(Clone, Debug, Default)]
pub struct LinearModelTrainOptions {
	pub l2_regularization: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_epochs: Option<u64>,
	pub n_examples_per_batch: Option<u64>,
	pub early_stopping_options: Option<EarlyStoppingOptions>,
}

#[derive(Clone, Debug, Default)]
pub struct TreeModelTrainOptions {
	pub binned_features_layout: Option<BinnedFeaturesLayout>,
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	pub l2_regularization_for_continuous_splits: Option<f32>,
	pub l2_regularization_for_discrete_splits: Option<f32>,
	pub learning_rate: Option<f32>,
	pub max_depth: Option<u64>,
	pub max_examples_for_computing_bin_thresholds: Option<u64>,
	pub max_leaf_nodes: Option<u64>,
	pub max_rounds: Option<u64>,
	pub max_valid_bins_for_number_features: Option<u8>,
	pub min_examples_per_node: Option<u64>,
	pub min_gain_to_split: Option<f32>,
	pub min_sum_hessians_per_node: Option<f32>,
	pub smoothing_factor_for_discrete_bin_sorting: Option<f32>,
}

#[derive(Clone, Debug)]
pub enum BinnedFeaturesLayout {
	RowMajor,
	ColumnMajor,
}

#[derive(Clone, Debug)]
pub struct EarlyStoppingOptions {
	pub early_stopping_fraction: f32,
	pub early_stopping_rounds: usize,
	pub early_stopping_threshold: f32,
}

impl Default for EarlyStoppingOptions {
	fn default() -> Self {
		EarlyStoppingOptions {
			early_stopping_fraction: 0.1,
			early_stopping_rounds: 5,
			early_stopping_threshold: 1e-5,
		}
	}
}

pub fn compute_regression_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearRegressor {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeRegressor {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					binned_features_layout: item.binned_features_layout.as_ref().map(
						|binned_feature_layout| match binned_feature_layout {
							config::BinnedFeaturesLayout::RowMajor => {
								BinnedFeaturesLayout::RowMajor
							}
							config::BinnedFeaturesLayout::ColumnMajor => {
								BinnedFeaturesLayout::ColumnMajor
							}
						},
					),
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
					l2_regularization_for_continuous_splits: item
						.l2_regularization_for_continuous_splits,
					l2_regularization_for_discrete_splits: item
						.l2_regularization_for_discrete_splits,
					learning_rate: item.learning_rate,
					max_depth: item.max_depth,
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_rounds: item.max_rounds,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_examples_per_node: item.min_examples_per_node,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
				},
			},
		})
		.collect()
}

pub fn compute_binary_classification_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearBinaryClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeBinaryClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					binned_features_layout: item.binned_features_layout.as_ref().map(
						|binned_feature_layout| match binned_feature_layout {
							config::BinnedFeaturesLayout::RowMajor => {
								BinnedFeaturesLayout::RowMajor
							}
							config::BinnedFeaturesLayout::ColumnMajor => {
								BinnedFeaturesLayout::ColumnMajor
							}
						},
					),
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
					l2_regularization_for_continuous_splits: item
						.l2_regularization_for_continuous_splits,
					l2_regularization_for_discrete_splits: item
						.l2_regularization_for_discrete_splits,
					learning_rate: item.learning_rate,
					max_depth: item.max_depth,
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_rounds: item.max_rounds,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_examples_per_node: item.min_examples_per_node,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
				},
			},
		})
		.collect()
}

pub fn compute_multiclass_classification_hyperparameter_grid(
	grid: &[config::GridItem],
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	grid.iter()
		.map(|item| match item {
			config::GridItem::Linear(item) => GridItem::LinearMulticlassClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: item.l2_regularization,
					learning_rate: item.learning_rate,
					max_epochs: item.max_epochs,
					n_examples_per_batch: item.n_examples_per_batch,
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
				},
			},
			config::GridItem::Tree(item) => GridItem::TreeMulticlassClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					binned_features_layout: item.binned_features_layout.as_ref().map(
						|binned_feature_layout| match binned_feature_layout {
							config::BinnedFeaturesLayout::RowMajor => {
								BinnedFeaturesLayout::RowMajor
							}
							config::BinnedFeaturesLayout::ColumnMajor => {
								BinnedFeaturesLayout::ColumnMajor
							}
						},
					),
					early_stopping_options: item.early_stopping_options.as_ref().map(
						|early_stopping_options| EarlyStoppingOptions {
							early_stopping_fraction: early_stopping_options.early_stopping_fraction,
							early_stopping_rounds: early_stopping_options
								.n_rounds_without_improvement_to_stop,
							early_stopping_threshold: early_stopping_options
								.min_decrease_in_loss_for_significant_change,
						},
					),
					l2_regularization_for_continuous_splits: item
						.l2_regularization_for_continuous_splits,
					l2_regularization_for_discrete_splits: item
						.l2_regularization_for_discrete_splits,
					learning_rate: item.learning_rate,
					max_depth: item.max_depth,
					max_examples_for_computing_bin_thresholds: item
						.max_examples_for_computing_bin_thresholds,
					max_leaf_nodes: item.max_leaf_nodes,
					max_rounds: item.max_rounds,
					max_valid_bins_for_number_features: item.max_valid_bins_for_number_features,
					min_examples_per_node: item.min_examples_per_node,
					min_gain_to_split: item.min_gain_to_split,
					min_sum_hessians_per_node: item.min_sum_hessians_per_node,
					smoothing_factor_for_discrete_bin_sorting: item
						.smoothing_factor_for_discrete_bin_sorting,
				},
			},
		})
		.collect()
}

const DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES: [f32; 2] = [0.1, 0.01];
const DEFAULT_LINEAR_L2_REGULARIZATION_VALUES: [f32; 2] = [1.0, 0.1];
const DEFAULT_LINEAR_MAX_EPOCHS_VALUES: [u64; 1] = [1000];
const DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES: [u64; 1] = [128];

const DEFAULT_TREE_LEARNING_RATE_VALUES: [f32; 2] = [0.1, 0.01];
const DEFAULT_TREE_L2_REGULARIZATION_VALUES_FOR_CONTINUOUS_SPLITS: [f32; 2] = [1.0, 0.1];
const DEFAULT_TREE_MAX_LEAF_NODES: [u64; 1] = [512];
const DEFAULT_TREE_MAX_ROUNDS_VALUES: [u64; 1] = [1000];
const DEFAULT_TREE_MAX_DEPTH: [u64; 1] = [50];

/// Compute the default hyperparameter grid for regression.
pub fn auto_regression_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	let autogrid = &config.train.autogrid;
	let (train_linear, train_tree) = match autogrid {
		Some(ag) => match &ag.model_types {
			Some(vec) => (
				vec.contains(&config::ModelType::Linear),
				vec.contains(&config::ModelType::Tree),
			),
			None => (true, true),
		},
		None => (true, true),
	};
	let mut grid = Vec::new();
	if train_linear {
		for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
			DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
			DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
			DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
			DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
		) {
			grid.push(GridItem::LinearRegressor {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: Some(l2_regularization),
					learning_rate: Some(learning_rate),
					max_epochs: Some(max_epochs),
					n_examples_per_batch: Some(n_examples_per_batch),
					early_stopping_options: Some(Default::default()),
				},
			});
		}
	}
	if train_tree {
		for (
			&max_leaf_nodes,
			&learning_rate,
			&l2_regularization_for_continuous_splits,
			&max_rounds,
			&max_depth,
		) in iproduct!(
			DEFAULT_TREE_MAX_LEAF_NODES.iter(),
			DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
			DEFAULT_TREE_L2_REGULARIZATION_VALUES_FOR_CONTINUOUS_SPLITS.iter(),
			DEFAULT_TREE_MAX_ROUNDS_VALUES.iter(),
			DEFAULT_TREE_MAX_DEPTH.iter()
		) {
			grid.push(GridItem::TreeRegressor {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					max_leaf_nodes: Some(max_leaf_nodes),
					learning_rate: Some(learning_rate),
					max_rounds: Some(max_rounds),
					max_depth: Some(max_depth),
					l2_regularization_for_continuous_splits: Some(
						l2_regularization_for_continuous_splits,
					),
					early_stopping_options: Some(Default::default()),
					..Default::default()
				},
			});
		}
	}
	grid
}

/// Compute the default hyperparameter grid for binary classification.
pub fn auto_binary_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	let autogrid = &config.train.autogrid;
	let (train_linear, train_tree) = match autogrid {
		Some(ag) => match &ag.model_types {
			Some(vec) => (
				vec.contains(&config::ModelType::Linear),
				vec.contains(&config::ModelType::Tree),
			),
			None => (true, true),
		},
		None => (true, true),
	};
	let mut grid = Vec::new();
	if train_linear {
		for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
			DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
			DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
			DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
			DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
		) {
			grid.push(GridItem::LinearBinaryClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: Some(l2_regularization),
					learning_rate: Some(learning_rate),
					max_epochs: Some(max_epochs),
					n_examples_per_batch: Some(n_examples_per_batch),
					early_stopping_options: Some(Default::default()),
				},
			});
		}
	}
	if train_tree {
		for (
			&max_leaf_nodes,
			&learning_rate,
			&l2_regularization_for_continous_splits,
			&max_rounds,
			&max_depth,
		) in iproduct!(
			DEFAULT_TREE_MAX_LEAF_NODES.iter(),
			DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
			DEFAULT_TREE_L2_REGULARIZATION_VALUES_FOR_CONTINUOUS_SPLITS.iter(),
			DEFAULT_TREE_MAX_ROUNDS_VALUES.iter(),
			DEFAULT_TREE_MAX_DEPTH.iter()
		) {
			grid.push(GridItem::TreeBinaryClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					max_leaf_nodes: Some(max_leaf_nodes),
					learning_rate: Some(learning_rate),
					max_rounds: Some(max_rounds),
					max_depth: Some(max_depth),
					l2_regularization_for_continuous_splits: Some(
						l2_regularization_for_continous_splits,
					),
					early_stopping_options: Some(Default::default()),
					..Default::default()
				},
			});
		}
	}
	grid
}

/// Compute the default hyperparameter grid for multiclass classification.
pub fn auto_multiclass_classification_hyperparameter_grid(
	target_column_index: usize,
	column_stats: &[ColumnStatsOutput],
	config: &config::Config,
) -> Vec<GridItem> {
	let autogrid = &config.train.autogrid;
	let (train_linear, train_tree) = match autogrid {
		Some(ag) => match &ag.model_types {
			Some(vec) => (
				vec.contains(&config::ModelType::Linear),
				vec.contains(&config::ModelType::Tree),
			),
			None => (true, true),
		},
		None => (true, true),
	};
	let mut grid = Vec::new();
	if train_linear {
		for (&l2_regularization, &learning_rate, &max_epochs, &n_examples_per_batch) in iproduct!(
			DEFAULT_LINEAR_L2_REGULARIZATION_VALUES.iter(),
			DEFAULT_LINEAR_MODEL_LEARNING_RATE_VALUES.iter(),
			DEFAULT_LINEAR_MAX_EPOCHS_VALUES.iter(),
			DEFAULT_LINEAR_N_EXAMPLES_PER_BATCH_VALUES.iter()
		) {
			grid.push(GridItem::LinearMulticlassClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_linear(column_stats, config),
				options: LinearModelTrainOptions {
					l2_regularization: Some(l2_regularization),
					learning_rate: Some(learning_rate),
					max_epochs: Some(max_epochs),
					n_examples_per_batch: Some(n_examples_per_batch),
					early_stopping_options: Some(Default::default()),
				},
			});
		}
	}
	if train_tree {
		for (
			&max_leaf_nodes,
			&learning_rate,
			&l2_regularization_for_continuous_splits,
			&max_rounds,
			&max_depth,
		) in iproduct!(
			DEFAULT_TREE_MAX_LEAF_NODES.iter(),
			DEFAULT_TREE_LEARNING_RATE_VALUES.iter(),
			DEFAULT_TREE_L2_REGULARIZATION_VALUES_FOR_CONTINUOUS_SPLITS.iter(),
			DEFAULT_TREE_MAX_ROUNDS_VALUES.iter(),
			DEFAULT_TREE_MAX_DEPTH.iter()
		) {
			grid.push(GridItem::TreeMulticlassClassifier {
				target_column_index,
				feature_groups: choose_feature_groups_tree(column_stats, config),
				options: TreeModelTrainOptions {
					max_leaf_nodes: Some(max_leaf_nodes),
					learning_rate: Some(learning_rate),
					max_rounds: Some(max_rounds),
					max_depth: Some(max_depth),
					l2_regularization_for_continuous_splits: Some(
						l2_regularization_for_continuous_splits,
					),
					early_stopping_options: Some(Default::default()),
					..Default::default()
				},
			});
		}
	}
	grid
}
