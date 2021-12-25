use crate::Progress;

use super::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train_early_stopping_split, EarlyStoppingMonitor, TrainOptions, TrainProgressEvent,
};
use ndarray::{self, prelude::*};
use num::{clamp, ToPrimitive};
use rayon::{self, prelude::*};
use std::num::NonZeroUsize;
use tangram_metrics::{CrossEntropy, CrossEntropyInput};
use tangram_progress_counter::ProgressCounter;
use tangram_table::prelude::*;
use tangram_zip::{pzip, zip};

/// This struct describes a linear multiclass classifier model. You can train one by calling `MulticlassClassifier::train`.
#[derive(Clone, Debug)]
pub struct MulticlassClassifier {
	/// These are the biases the model learned.
	pub biases: Array1<f32>,
	/// These are the weights the model learned. The shape is (n_features, n_classes).
	pub weights: Array2<f32>,
	/// These are the mean values of each feature in the training set. They are used to compute SHAP values.
	pub means: Vec<f32>,
}

/// This struct is returned by `MulticlassClassifier::train`.
pub struct MulticlassClassifierTrainOutput {
	/// This is the model you just trained.
	pub model: MulticlassClassifier,
	/// These are the loss values for each epoch.
	pub losses: Option<Vec<f32>>,
	/// These are the importances of each feature.
	pub feature_importances: Option<Vec<f32>>,
}

impl MulticlassClassifier {
	/// Train a linear multiclass classifier.
	pub fn train(
		features: ArrayView2<f32>,
		labels: EnumTableColumnView,
		train_options: &TrainOptions,
		progress: Progress,
	) -> MulticlassClassifierTrainOutput {
		let n_classes = labels.variants().len();
		let n_features = features.ncols();
		let (features_train, labels_train, features_early_stopping, labels_early_stopping) =
			train_early_stopping_split(
				features,
				labels.as_slice().into(),
				train_options
					.early_stopping_options
					.as_ref()
					.map(|o| o.early_stopping_fraction)
					.unwrap_or(0.0),
			);
		let means = features_train
			.axis_iter(Axis(1))
			.map(|column| column.mean().unwrap())
			.collect();
		let mut model = MulticlassClassifier {
			biases: <Array1<f32>>::zeros(n_classes),
			weights: <Array2<f32>>::zeros((n_features, n_classes)),
			means,
		};
		let mut early_stopping_monitor =
			train_options
				.early_stopping_options
				.as_ref()
				.map(|early_stopping_options| {
					EarlyStoppingMonitor::new(
						early_stopping_options.min_decrease_in_loss_for_significant_change,
						early_stopping_options.n_rounds_without_improvement_to_stop,
					)
				});
		let progress_counter = ProgressCounter::new(train_options.max_epochs.to_u64().unwrap());
		(progress.handle_progress_event)(TrainProgressEvent::Train(progress_counter.clone()));
		let mut probabilities_buffer: Array2<f32> = Array2::zeros((labels.len(), n_classes));
		let mut losses = if train_options.compute_losses {
			Some(Vec::new())
		} else {
			None
		};
		let kill_chip = progress.kill_chip;
		for _ in 0..train_options.max_epochs {
			progress_counter.inc(1);
			let n_examples_per_batch = train_options.n_examples_per_batch;
			struct MulticlassClassifierPtr(*mut MulticlassClassifier);
			unsafe impl Send for MulticlassClassifierPtr {}
			unsafe impl Sync for MulticlassClassifierPtr {}
			let model_ptr = MulticlassClassifierPtr(&mut model);
			pzip!(
				features_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
				probabilities_buffer.axis_chunks_iter_mut(Axis(0), n_examples_per_batch),
			)
			.for_each(|(features, labels, probabilities)| {
				let model_ptr = &model_ptr;
				let model = unsafe { &mut *model_ptr.0 };
				MulticlassClassifier::train_batch(
					model,
					features,
					labels,
					probabilities,
					train_options,
					kill_chip,
				);
			});
			if let Some(losses) = &mut losses {
				let loss =
					MulticlassClassifier::compute_loss(probabilities_buffer.view(), labels_train);
				losses.push(loss);
			}
			if let Some(early_stopping_monitor) = early_stopping_monitor.as_mut() {
				let early_stopping_metric_value =
					MulticlassClassifier::compute_early_stopping_metric_value(
						&model,
						features_early_stopping,
						labels_early_stopping,
						train_options,
					);
				let should_stop = early_stopping_monitor.update(early_stopping_metric_value);
				if should_stop {
					break;
				}
			}
			// Check if we should stop training.
			if progress.kill_chip.is_activated() {
				break;
			}
		}
		(progress.handle_progress_event)(TrainProgressEvent::TrainDone);
		let feature_importances = MulticlassClassifier::compute_feature_importances(&model);
		MulticlassClassifierTrainOutput {
			model,
			losses,
			feature_importances: Some(feature_importances),
		}
	}

	fn compute_feature_importances(model: &MulticlassClassifier) -> Vec<f32> {
		// Compute the absolute value of each of the weights.
		let mut feature_importances = model
			.weights
			.axis_iter(Axis(0))
			.map(|weights_each_class| {
				weights_each_class
					.iter()
					.map(|weight| weight.abs())
					.sum::<f32>() / model.weights.ncols().to_f32().unwrap()
			})
			.collect::<Vec<_>>();
		// Compute the sum and normalize so the importances sum to 1.
		let feature_importances_sum: f32 = feature_importances.iter().sum::<f32>();
		feature_importances
			.iter_mut()
			.for_each(|feature_importance| *feature_importance /= feature_importances_sum);
		feature_importances
	}

	fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		mut probabilities: ArrayViewMut2<f32>,
		train_options: &TrainOptions,
		kill_chip: &tangram_kill_chip::KillChip,
	) {
		if kill_chip.is_activated() {
			return;
		}
		let learning_rate = train_options.learning_rate;
		let n_classes = self.weights.ncols();
		let mut logits = features.dot(&self.weights) + &self.biases;
		softmax(logits.view_mut());
		for (probability, logit) in zip!(probabilities.iter_mut(), logits.iter()) {
			*probability = *logit;
		}
		let mut predictions = logits;
		for (mut predictions, label) in zip!(predictions.axis_iter_mut(Axis(0)), labels) {
			for (class_index, prediction) in predictions.iter_mut().enumerate() {
				*prediction -= if class_index == label.unwrap().get() - 1 {
					1.0
				} else {
					0.0
				};
			}
		}
		let py = predictions;
		for class_index in 0..n_classes {
			let weight_gradients = (&features * &py.column(class_index).insert_axis(Axis(1)))
				.mean_axis(Axis(0))
				.unwrap();
			for (weight, weight_gradient) in zip!(
				self.weights.column_mut(class_index),
				weight_gradients.iter()
			) {
				*weight += -learning_rate * weight_gradient
			}
			let bias_gradients = py
				.column(class_index)
				.insert_axis(Axis(1))
				.mean_axis(Axis(0))
				.unwrap();
			self.biases[class_index] += -learning_rate * bias_gradients[0];
		}
	}

	pub fn compute_loss(
		probabilities: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
	) -> f32 {
		let mut loss = 0.0;
		for (label, probabilities) in zip!(labels.into_iter(), probabilities.axis_iter(Axis(0))) {
			for (index, &probability) in probabilities.indexed_iter() {
				let probability = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
				if index == (label.unwrap().get() - 1) {
					loss += -probability.ln();
				}
			}
		}
		loss / labels.len().to_f32().unwrap()
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		train_options: &TrainOptions,
	) -> f32 {
		let n_classes = self.biases.len();
		pzip!(
			features.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
		)
		.fold(
			|| {
				let predictions = unsafe {
					<Array2<f32>>::uninit((train_options.n_examples_per_batch, n_classes))
						.assume_init()
				};
				let metric = CrossEntropy::default();
				(predictions, metric)
			},
			|(mut predictions, mut metric), (features, labels)| {
				let slice = s![0..features.nrows(), ..];
				let mut predictions_slice = predictions.slice_mut(slice);
				self.predict(features, predictions_slice.view_mut());
				for (prediction, label) in zip!(predictions_slice.axis_iter(Axis(0)), labels.iter())
				{
					metric.update(CrossEntropyInput {
						probabilities: prediction,
						label: *label,
					});
				}
				(predictions, metric)
			},
		)
		.map(|(_, metric)| metric)
		.reduce(CrossEntropy::new, |mut a, b| {
			a.merge(b);
			a
		})
		.finalize()
		.0
		.unwrap()
	}

	/// Write predicted probabilities into `probabilities` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut2<f32>) {
		for mut row in probabilities.axis_iter_mut(Axis(0)) {
			row.assign(&self.biases.view());
		}
		ndarray::linalg::general_mat_mul(1.0, &features, &self.weights, 1.0, &mut probabilities);
		softmax(probabilities);
	}

	pub fn compute_feature_contributions(
		&self,
		features: ArrayView2<f32>,
	) -> Vec<Vec<ComputeShapValuesForExampleOutput>> {
		features
			.axis_iter(Axis(0))
			.map(|features| {
				zip!(self.weights.axis_iter(Axis(1)), self.biases.view())
					.map(|(weights, bias)| {
						compute_shap_values_for_example(
							features.as_slice().unwrap(),
							*bias,
							weights.view(),
							&self.means,
						)
					})
					.collect()
			})
			.collect()
	}

	pub fn from_reader(
		multiclass_classifier: crate::serialize::MulticlassClassifierReader,
	) -> MulticlassClassifier {
		crate::serialize::deserialize_multiclass_classifier(multiclass_classifier)
	}

	pub fn to_writer(
		&self,
		writer: &mut buffalo::Writer,
	) -> buffalo::Position<crate::serialize::MulticlassClassifierWriter> {
		crate::serialize::serialize_multiclass_classifier(self, writer)
	}

	#[must_use]
	pub fn from_bytes(&self, bytes: &[u8]) -> MulticlassClassifier {
		let reader = buffalo::read::<crate::serialize::MulticlassClassifierReader>(bytes);
		Self::from_reader(reader)
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		// Create the writer.
		let mut writer = buffalo::Writer::new();
		self.to_writer(&mut writer);
		writer.into_bytes()
	}
}

fn softmax(mut logits: ArrayViewMut2<f32>) {
	for mut logits in logits.axis_iter_mut(Axis(0)) {
		let max = logits.iter().fold(std::f32::MIN, |a, &b| f32::max(a, b));
		for logit in logits.iter_mut() {
			*logit = (*logit - max).exp();
		}
		let sum = logits.iter().sum::<f32>();
		for logit in logits.iter_mut() {
			*logit /= sum;
		}
	}
}
