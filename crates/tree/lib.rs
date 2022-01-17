/*!
This crate implements machine learning models for regression and classification using ensembles of decision trees. It has many similarities to [LightGBM](github.com/microsoft/lightgbm), [XGBoost](github.com/xgboost/xgboost), and others, but is written in pure Rust.

For an example of regression, see `benchmarks/boston.rs`.rs`. For an example of binary classification, see `benchmarks/heart_disease.rs`. For an example of multiclass classification, see `benchmarks/iris.rs`.
*/

pub use self::{
	binary_classifier::{BinaryClassifier, BinaryClassifierTrainOutput},
	multiclass_classifier::{MulticlassClassifier, MulticlassClassifierTrainOutput},
	regressor::{Regressor, RegressorTrainOutput},
};
use bitvec::prelude::*;
use tangram_progress_counter::ProgressCounter;

mod binary_classifier;
mod choose_best_split;
mod compute_bin_stats;
mod compute_binned_features;
mod compute_binning_instructions;
mod compute_feature_importances;
mod multiclass_classifier;
mod pool;
mod rearrange_examples_index;
mod regressor;
pub mod serialize;
mod shap;
#[cfg(feature = "timing")]
mod timing;
mod train;
mod train_tree;

pub struct Progress<'a> {
	pub kill_chip: &'a tangram_kill_chip::KillChip,
	pub handle_progress_event: &'a mut dyn FnMut(TrainProgressEvent),
}

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Clone, Debug)]
pub struct TrainOptions {
	/// This option controls whether binned features will be laid out in row major or column major order. Each will produce the same result, but row major will be faster for datasets with more rows and fewer columns, while column major will be faster for datasets with fewer rows and more columns.
	pub binned_features_layout: BinnedFeaturesLayout,
	/// If true, the model will include the loss on the training data after each round.
	pub compute_losses: bool,
	/// This option controls early stopping. If it is `Some`, then early stopping will be enabled. If it is `None`, then early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This option sets the L2 regularization value for continuous splits, which helps avoid overfitting.
	pub l2_regularization_for_continuous_splits: f32,
	/// This option sets the L2 regularization value for discrete splits, which helps avoid overfitting.
	pub l2_regularization_for_discrete_splits: f32,
	/// The learning rate scales the leaf values to control the effect each tree has on the output.
	pub learning_rate: f32,
	/// This is the maximum depth of a single tree. If this value is `None`, the depth will not be limited.
	pub max_depth: Option<usize>,
	/// This is the maximum number of examples to consider when determining the bin thresholds for number features.
	pub max_examples_for_computing_bin_thresholds: usize,
	/// This is the maximum number of leaf nodes in a single tree.
	pub max_leaf_nodes: usize,
	/// This is the maximum number of rounds of training that will occur. Fewer rounds may be trained if early stopping is enabled.
	pub max_rounds: usize,
	/// When computing the bin thresholds for number features, this is the maximum number of bins for valid values to create. If the number of unique values in the number feature is less than this value, the thresholds will be equal to the unique values, which can improve accuracy when number features have a small set of possible values.
	pub max_valid_bins_for_number_features: u8,
	/// A split will only be considered valid if the number of training examples sent to each of the resulting children is at least this value.
	pub min_examples_per_node: usize,
	/// A node will only be split if the best split achieves at least this minimum gain.
	pub min_gain_to_split: f32,
	/// A split will only be considered valid if the sum of hessians in each of the resulting children is at least this value.
	pub min_sum_hessians_per_node: f32,
	/// When choosing which direction each enum variant should be sent in a discrete split, the enum variants are sorted by a score computed from the sum of gradients and hessians for examples with that enum variant. This smoothing factor is added to the denominator of that score.
	pub smoothing_factor_for_discrete_bin_sorting: f32,
}

impl Default for TrainOptions {
	fn default() -> TrainOptions {
		TrainOptions {
			binned_features_layout: BinnedFeaturesLayout::ColumnMajor,
			compute_losses: false,
			early_stopping_options: None,
			l2_regularization_for_continuous_splits: 0.0,
			l2_regularization_for_discrete_splits: 10.0,
			learning_rate: 0.1,
			max_depth: None,
			max_leaf_nodes: 31,
			max_rounds: 100,
			max_valid_bins_for_number_features: 255,
			min_examples_per_node: 20,
			min_gain_to_split: 0.0,
			min_sum_hessians_per_node: 1e-3,
			max_examples_for_computing_bin_thresholds: 200_000,
			smoothing_factor_for_discrete_bin_sorting: 10.0,
		}
	}
}

/// This enum defines whether binned features will be layed out in row major or column major order.
#[derive(Clone, Copy, Debug)]
pub enum BinnedFeaturesLayout {
	RowMajor,
	ColumnMajor,
}

/// The parameters in this struct control how to determine whether training should stop early after each round or epoch.
#[derive(Clone, Debug)]
pub struct EarlyStoppingOptions {
	/// This is the fraction of the dataset that is set aside to compute the early stopping metric.
	pub early_stopping_fraction: f32,
	/// If this many rounds or epochs pass by without a significant improvement in the early stopping metric over the previous round or epoch, training will be stopped early.
	pub n_rounds_without_improvement_to_stop: usize,
	/// This is the minimum descrease in the early stopping metric for a round or epoch to be considered a significant improvement over the previous round or epoch.
	pub min_decrease_in_loss_for_significant_change: f32,
}

/// This struct describes the training progress.
#[derive(Clone, Debug)]
pub enum TrainProgressEvent {
	Initialize(ProgressCounter),
	InitializeDone,
	Train(ProgressCounter),
	TrainDone,
}

/// Trees are stored as a `Vec` of `Node`s. Each branch in the tree has two indexes into the `Vec`, one for each of its children.
#[derive(Clone, Debug)]
pub struct Tree {
	pub nodes: Vec<Node>,
}

impl Tree {
	/// Make a prediction.
	pub fn predict(&self, example: &[tangram_table::TableValue]) -> f32 {
		// Start at the root node.
		let mut node_index = 0;
		// Traverse the tree until we get to a leaf.
		unsafe {
			loop {
				match self.nodes.get_unchecked(node_index) {
					// We made it to a leaf! The prediction is the leaf's value.
					Node::Leaf(LeafNode { value, .. }) => return *value as f32,
					// This branch uses a continuous split.
					Node::Branch(BranchNode {
						left_child_index,
						right_child_index,
						split:
							BranchSplit::Continuous(BranchSplitContinuous {
								feature_index,
								split_value,
								..
							}),
						..
					}) => {
						node_index = if example.get_unchecked(*feature_index).as_number().unwrap()
							<= split_value
						{
							*left_child_index
						} else {
							*right_child_index
						};
					}
					// This branch uses a discrete split.
					Node::Branch(BranchNode {
						left_child_index,
						right_child_index,
						split:
							BranchSplit::Discrete(BranchSplitDiscrete {
								feature_index,
								directions,
								..
							}),
						..
					}) => {
						let bin_index = if let Some(bin_index) =
							example.get_unchecked(*feature_index).as_enum().unwrap()
						{
							bin_index.get()
						} else {
							0
						};
						let direction = (*directions.get(bin_index).unwrap()).into();
						node_index = match direction {
							SplitDirection::Left => *left_child_index,
							SplitDirection::Right => *right_child_index,
						};
					}
				}
			}
		}
	}
}

/// A node is either a branch or a leaf.
#[derive(Clone, Debug)]
pub enum Node {
	Branch(BranchNode),
	Leaf(LeafNode),
}

impl Node {
	pub fn as_branch(&self) -> Option<&BranchNode> {
		match self {
			Node::Branch(branch) => Some(branch),
			_ => None,
		}
	}

	pub fn as_leaf(&self) -> Option<&LeafNode> {
		match self {
			Node::Leaf(leaf) => Some(leaf),
			_ => None,
		}
	}

	pub fn examples_fraction(&self) -> f32 {
		match self {
			Node::Leaf(LeafNode {
				examples_fraction, ..
			}) => *examples_fraction,
			Node::Branch(BranchNode {
				examples_fraction, ..
			}) => *examples_fraction,
		}
	}
}

/// A `BranchNode` is a branch in a tree.
#[derive(Clone, Debug)]
pub struct BranchNode {
	/// This is the index in the tree's node vector for this node's left child.
	pub left_child_index: usize,
	/// This is the index in the tree's node vector for this node's right child.
	pub right_child_index: usize,
	/// When making predictions, an example will be sent either to the right or left child. The `split` contains the information necessary to determine which way it will go.
	pub split: BranchSplit,
	/// Branch nodes store the fraction of training examples that passed through them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

/// A `BranchSplit` describes how examples are sent to the left or right child given their feature values. A `Continous` split is used for number features, and `Discrete` is used for enum features.
#[derive(Clone, Debug)]
pub enum BranchSplit {
	Continuous(BranchSplitContinuous),
	Discrete(BranchSplitDiscrete),
}

/// A continuous branch split takes the value of a single number feature, compares it with a `split_value`, and if the value is <= `split_value`, the example is sent left, and if it is > `split_value`, it is sent right.
#[derive(Clone, Debug)]
pub struct BranchSplitContinuous {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// This is the threshold value of the split.
	pub split_value: f32,
	/// This is the direction invalid values should be sent.
	pub invalid_values_direction: SplitDirection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitDirection {
	Left,
	Right,
}

impl From<bool> for SplitDirection {
	fn from(value: bool) -> Self {
		match value {
			false => SplitDirection::Left,
			true => SplitDirection::Right,
		}
	}
}

impl From<SplitDirection> for bool {
	fn from(value: SplitDirection) -> Self {
		match value {
			SplitDirection::Left => false,
			SplitDirection::Right => true,
		}
	}
}

/// A discrete branch split takes the value of a single enum feature and looks up in a bitset which way the example should be sent.
#[derive(Clone, Debug)]
pub struct BranchSplitDiscrete {
	/// This is the index of the feature to get the value for.
	pub feature_index: usize,
	/// This specifies which direction, left or right, an example should be sent, based on the value of the chosen feature.
	pub directions: BitVec<u8, Lsb0>,
}

/// The leaves in a tree hold the values to output for examples that get sent to them.
#[derive(Clone, Debug)]
pub struct LeafNode {
	/// This is the value to output.
	pub value: f64,
	/// Leaf nodes store the fraction of training examples that were sent to them during training. This is used to compute SHAP values.
	pub examples_fraction: f32,
}

impl BranchSplit {
	pub fn feature_index(&self) -> usize {
		match self {
			BranchSplit::Continuous(s) => s.feature_index,
			BranchSplit::Discrete(s) => s.feature_index,
		}
	}
}
