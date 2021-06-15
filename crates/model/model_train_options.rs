#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum ModelTrainOptions {
	#[buffalo(id = 0, required)]
	Linear(LinearModelTrainOptions),
	#[buffalo(id = 1, required)]
	Tree(TreeModelTrainOptions),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LinearModelTrainOptions {
	#[buffalo(id = 0, required)]
	pub compute_loss: bool,
	#[buffalo(id = 1, required)]
	pub l2_regularization: f32,
	#[buffalo(id = 2, required)]
	pub learning_rate: f32,
	#[buffalo(id = 3, required)]
	pub max_epochs: u64,
	#[buffalo(id = 4, required)]
	pub n_examples_per_batch: u64,
	#[buffalo(id = 5, required)]
	pub early_stopping_options: Option<LinearEarlyStoppingOptions>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LinearEarlyStoppingOptions {
	#[buffalo(id = 0, required)]
	pub early_stopping_fraction: f32,
	#[buffalo(id = 1, required)]
	pub n_rounds_without_improvement_to_stop: u64,
	#[buffalo(id = 2, required)]
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TreeModelTrainOptions {
	#[buffalo(id = 0, required)]
	pub binned_features_layout: BinnedFeaturesLayout,
	#[buffalo(id = 1, required)]
	pub compute_loss: bool,
	#[buffalo(id = 2, required)]
	pub early_stopping_options: Option<TreeEarlyStoppingOptions>,
	#[buffalo(id = 3, required)]
	pub l2_regularization_for_continuous_splits: f32,
	#[buffalo(id = 4, required)]
	pub l2_regularization_for_discrete_splits: f32,
	#[buffalo(id = 5, required)]
	pub learning_rate: f32,
	#[buffalo(id = 6, required)]
	pub max_depth: Option<u64>,
	#[buffalo(id = 7, required)]
	pub max_examples_for_computing_bin_thresholds: u64,
	#[buffalo(id = 8, required)]
	pub max_leaf_nodes: u64,
	#[buffalo(id = 9, required)]
	pub max_rounds: u64,
	#[buffalo(id = 10, required)]
	pub max_valid_bins_for_number_features: u8,
	#[buffalo(id = 11, required)]
	pub min_examples_per_node: u64,
	#[buffalo(id = 12, required)]
	pub min_gain_to_split: f32,
	#[buffalo(id = 13, required)]
	pub min_sum_hessians_per_node: f32,
	#[buffalo(id = 14, required)]
	pub smoothing_factor_for_discrete_bin_sorting: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum BinnedFeaturesLayout {
	#[buffalo(id = 0)]
	RowMajor,
	#[buffalo(id = 1)]
	ColumnMajor,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TreeEarlyStoppingOptions {
	#[buffalo(id = 0, required)]
	pub early_stopping_fraction: f32,
	#[buffalo(id = 1, required)]
	pub n_rounds_without_improvement_to_stop: u64,
	#[buffalo(id = 2, required)]
	pub min_decrease_in_loss_for_significant_change: f32,
}
