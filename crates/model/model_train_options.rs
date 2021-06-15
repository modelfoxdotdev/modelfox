#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum ModelTrainOptions {
	#[tangram_serialize(id = 0, required)]
	Linear(LinearModelTrainOptions),
	#[tangram_serialize(id = 1, required)]
	Tree(TreeModelTrainOptions),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct LinearModelTrainOptions {
	#[tangram_serialize(id = 0, required)]
	pub compute_loss: bool,
	#[tangram_serialize(id = 1, required)]
	pub l2_regularization: f32,
	#[tangram_serialize(id = 2, required)]
	pub learning_rate: f32,
	#[tangram_serialize(id = 3, required)]
	pub max_epochs: u64,
	#[tangram_serialize(id = 4, required)]
	pub n_examples_per_batch: u64,
	#[tangram_serialize(id = 5, required)]
	pub early_stopping_options: Option<LinearEarlyStoppingOptions>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct LinearEarlyStoppingOptions {
	#[tangram_serialize(id = 0, required)]
	pub early_stopping_fraction: f32,
	#[tangram_serialize(id = 1, required)]
	pub n_rounds_without_improvement_to_stop: u64,
	#[tangram_serialize(id = 2, required)]
	pub min_decrease_in_loss_for_significant_change: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TreeModelTrainOptions {
	#[tangram_serialize(id = 0, required)]
	pub binned_features_layout: BinnedFeaturesLayout,
	#[tangram_serialize(id = 1, required)]
	pub compute_loss: bool,
	#[tangram_serialize(id = 2, required)]
	pub early_stopping_options: Option<TreeEarlyStoppingOptions>,
	#[tangram_serialize(id = 3, required)]
	pub l2_regularization_for_continuous_splits: f32,
	#[tangram_serialize(id = 4, required)]
	pub l2_regularization_for_discrete_splits: f32,
	#[tangram_serialize(id = 5, required)]
	pub learning_rate: f32,
	#[tangram_serialize(id = 6, required)]
	pub max_depth: Option<u64>,
	#[tangram_serialize(id = 7, required)]
	pub max_examples_for_computing_bin_thresholds: u64,
	#[tangram_serialize(id = 8, required)]
	pub max_leaf_nodes: u64,
	#[tangram_serialize(id = 9, required)]
	pub max_rounds: u64,
	#[tangram_serialize(id = 10, required)]
	pub max_valid_bins_for_number_features: u8,
	#[tangram_serialize(id = 11, required)]
	pub min_examples_per_node: u64,
	#[tangram_serialize(id = 12, required)]
	pub min_gain_to_split: f32,
	#[tangram_serialize(id = 13, required)]
	pub min_sum_hessians_per_node: f32,
	#[tangram_serialize(id = 14, required)]
	pub smoothing_factor_for_discrete_bin_sorting: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum BinnedFeaturesLayout {
	#[tangram_serialize(id = 0)]
	RowMajor,
	#[tangram_serialize(id = 1)]
	ColumnMajor,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TreeEarlyStoppingOptions {
	#[tangram_serialize(id = 0, required)]
	pub early_stopping_fraction: f32,
	#[tangram_serialize(id = 1, required)]
	pub n_rounds_without_improvement_to_stop: u64,
	#[tangram_serialize(id = 2, required)]
	pub min_decrease_in_loss_for_significant_change: f32,
}
