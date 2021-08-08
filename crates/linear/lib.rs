#![doc = include_str!("./README.md")]

pub use self::{
	binary_classifier::BinaryClassifier, multiclass_classifier::MulticlassClassifier,
	regressor::Regressor,
};
use ndarray::prelude::*;
use num::ToPrimitive;
use tangram_progress_counter::ProgressCounter;

mod binary_classifier;
mod multiclass_classifier;
mod regressor;
pub mod serialize;
mod shap;

/// These are the options passed to `Regressor::train`, `BinaryClassifier::train`, and `MulticlassClassifier::train`.
#[derive(Clone, Debug)]
pub struct TrainOptions {
	/// If true, the model will include the loss on the training data after each epoch.
	pub compute_losses: bool,
	/// Specify options for early stopping. If the value is `Some`, early stopping will be enabled. If it is `None`, early stopping will be disabled.
	pub early_stopping_options: Option<EarlyStoppingOptions>,
	/// This is the L2 regularization value to use when updating the model parameters.
	pub l2_regularization: f32,
	/// This is the learning rate to use when updating the model parameters.
	pub learning_rate: f32,
	/// This is the maximum number of epochs to train.
	pub max_epochs: usize,
	/// This is the number of examples to use for each batch of training.
	pub n_examples_per_batch: usize,
}

impl Default for TrainOptions {
	fn default() -> TrainOptions {
		TrainOptions {
			compute_losses: false,
			early_stopping_options: None,
			l2_regularization: 0.0,
			learning_rate: 0.1,
			max_epochs: 100,
			n_examples_per_batch: 32,
		}
	}
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

pub struct Progress<'a> {
	pub kill_chip: &'a tangram_kill_chip::KillChip,
	pub handle_progress_event: &'a mut dyn FnMut(TrainProgressEvent),
}

/// This is the training progress, which tracks the current epoch.
#[derive(Clone, Debug)]
pub enum TrainProgressEvent {
	Train(ProgressCounter),
	TrainDone,
}

/// This function splits the `features` and `labels` arrays into training and early stopping arrays, where the size of the early stopping stopping array will be `features.len() * early_stopping_fraction`.
fn train_early_stopping_split<'features, 'labels, Label>(
	features: ArrayView2<'features, f32>,
	labels: ArrayView1<'labels, Label>,
	early_stopping_fraction: f32,
) -> (
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
	ArrayView2<'features, f32>,
	ArrayView1<'labels, Label>,
) {
	let split_index = ((1.0 - early_stopping_fraction) * features.nrows().to_f32().unwrap())
		.to_usize()
		.unwrap();
	let (features_train, features_early_stopping) = features.split_at(Axis(0), split_index);
	let (labels_train, labels_early_stopping) = labels.split_at(Axis(0), split_index);
	(
		features_train,
		labels_train,
		features_early_stopping,
		labels_early_stopping,
	)
}

/**
The `EarlyStoppingMonitor` keeps track of the values of an early stopping metric for each epoch, and if enough epochs have passed without a significant improvement in the metric, the `update()` function will return `true` to indicate that training should be stopped.
*/
struct EarlyStoppingMonitor {
	threshold: f32,
	epochs: usize,
	n_epochs_without_observed_improvement: usize,
	previous_epoch_metric_value: Option<f32>,
}

impl EarlyStoppingMonitor {
	// Create a new `EarlyStoppingMonitor`.
	pub fn new(threshold: f32, epochs: usize) -> EarlyStoppingMonitor {
		EarlyStoppingMonitor {
			threshold,
			epochs,
			previous_epoch_metric_value: None,
			n_epochs_without_observed_improvement: 0,
		}
	}

	/// This function updates the `EarlyStoppingMonitor` with the next epoch's early stopping metric. THis function returns true if training should stop.
	pub fn update(&mut self, early_stopping_metric_value: f32) -> bool {
		let result = if let Some(previous_stopping_metric) = self.previous_epoch_metric_value {
			if early_stopping_metric_value > previous_stopping_metric
				|| f32::abs(early_stopping_metric_value - previous_stopping_metric) < self.threshold
			{
				self.n_epochs_without_observed_improvement += 1;
				self.n_epochs_without_observed_improvement >= self.epochs
			} else {
				self.n_epochs_without_observed_improvement = 0;
				false
			}
		} else {
			false
		};
		self.previous_epoch_metric_value = Some(early_stopping_metric_value);
		result
	}
}
