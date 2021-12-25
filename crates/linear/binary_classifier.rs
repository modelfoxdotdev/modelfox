use super::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train_early_stopping_split, EarlyStoppingMonitor, Progress, TrainOptions, TrainProgressEvent,
};
use ndarray::{self, prelude::*};
use num::{clamp, ToPrimitive};
use rayon::{self, prelude::*};
use std::{num::NonZeroUsize, ops::Neg};
use tangram_metrics::{BinaryCrossEntropy, BinaryCrossEntropyInput};
use tangram_progress_counter::ProgressCounter;
use tangram_table::prelude::*;
use tangram_zip::{pzip, zip};

/// This struct describes a linear binary classifier model. You can train one by calling `BinaryClassifier::train`.
#[derive(Clone, Debug)]
pub struct BinaryClassifier {
	/// This is the bias the model learned.
	pub bias: f32,
	/// These are the weights the model learned.
	pub weights: Array1<f32>,
	/// These are the mean values of each feature in the training set. They are used to compute SHAP values.
	pub means: Vec<f32>,
}

/// This struct is returned by `BinaryClassifier::train`.
pub struct BinaryClassifierTrainOutput {
	/// This is the model you just trained.
	pub model: BinaryClassifier,
	/// These are the loss values for each epoch.
	pub losses: Option<Vec<f32>>,
	/// These are the importances of each feature.
	pub feature_importances: Option<Vec<f32>>,
}

impl BinaryClassifier {
	/// Train a linear binary classifier.
	pub fn train(
		features: ArrayView2<f32>,
		labels: EnumTableColumnView,
		train_options: &TrainOptions,
		progress: Progress,
	) -> BinaryClassifierTrainOutput {
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
		let mut model = BinaryClassifier {
			bias: 0.0,
			weights: <Array1<f32>>::zeros(n_features),
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
		let mut probabilities_buffer: Array1<f32> = Array1::zeros(labels.len());
		let mut losses = if train_options.compute_losses {
			Some(Vec::new())
		} else {
			None
		};
		let kill_chip = progress.kill_chip;
		for _ in 0..train_options.max_epochs {
			progress_counter.inc(1);
			let n_examples_per_batch = train_options.n_examples_per_batch;
			struct BinaryClassifierPtr(*mut BinaryClassifier);
			unsafe impl Send for BinaryClassifierPtr {}
			unsafe impl Sync for BinaryClassifierPtr {}
			let model_ptr = BinaryClassifierPtr(&mut model);
			pzip!(
				features_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
				labels_train.axis_chunks_iter(Axis(0), n_examples_per_batch),
				probabilities_buffer.axis_chunks_iter_mut(Axis(0), n_examples_per_batch),
			)
			.for_each(|(features, labels, probabilities)| {
				let model_ptr = &model_ptr;
				let model = unsafe { &mut *model_ptr.0 };
				BinaryClassifier::train_batch(
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
					BinaryClassifier::compute_loss(probabilities_buffer.view(), labels_train);
				losses.push(loss);
			}
			if let Some(early_stopping_monitor) = early_stopping_monitor.as_mut() {
				let early_stopping_metric_value =
					BinaryClassifier::compute_early_stopping_metric_value(
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
		let feature_importances = BinaryClassifier::compute_feature_importances(&model);
		BinaryClassifierTrainOutput {
			model,
			losses,
			feature_importances: Some(feature_importances),
		}
	}

	fn compute_feature_importances(model: &BinaryClassifier) -> Vec<f32> {
		// Compute the absolute value of each of the weights.
		let mut feature_importances = model
			.weights
			.iter()
			.map(|weight| weight.abs())
			.collect::<Vec<_>>();
		// Compute the sum and normalize so the importances sum to 1.
		let feature_importances_sum = feature_importances.iter().sum::<f32>();
		feature_importances
			.iter_mut()
			.for_each(|feature_importance| *feature_importance /= feature_importances_sum);
		feature_importances
	}

	fn train_batch(
		&mut self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		mut probabilities: ArrayViewMut1<f32>,
		train_options: &TrainOptions,
		kill_chip: &tangram_kill_chip::KillChip,
	) {
		if kill_chip.is_activated() {
			return;
		}
		let learning_rate = train_options.learning_rate;
		let mut py = features.dot(&self.weights) + self.bias;
		for (probability, py) in zip!(probabilities.iter_mut(), py.iter_mut()) {
			*probability = 1.0 / (py.neg().exp() + 1.0);
			*py = *probability;
		}
		for (py, label) in zip!(py.view_mut(), labels) {
			let label = match label.map(|l| l.get()) {
				Some(1) => 0.0,
				Some(2) => 1.0,
				_ => unreachable!(),
			};
			*py -= label
		}
		let py = py.insert_axis(Axis(1));
		let weight_gradients = (&features * &py).mean_axis(Axis(0)).unwrap();
		let bias_gradient = py.mean_axis(Axis(0)).unwrap()[0];
		for (weight, weight_gradient) in zip!(self.weights.view_mut(), weight_gradients.view()) {
			*weight += -learning_rate * weight_gradient;
		}
		self.bias += -learning_rate * bias_gradient;
	}

	pub fn compute_loss(
		probabilities: ArrayView1<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
	) -> f32 {
		let mut total = 0.0;
		for (label, probability) in zip!(labels.iter(), probabilities) {
			let label = (label.unwrap().get() - 1).to_f32().unwrap();
			let probability_clamped =
				clamp(*probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
			total += -1.0 * label * probability_clamped.ln()
				+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln()
		}
		total / labels.len().to_f32().unwrap()
	}

	fn compute_early_stopping_metric_value(
		&self,
		features: ArrayView2<f32>,
		labels: ArrayView1<Option<NonZeroUsize>>,
		train_options: &TrainOptions,
	) -> f32 {
		pzip!(
			features.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
			labels.axis_chunks_iter(Axis(0), train_options.n_examples_per_batch),
		)
		.fold(
			|| {
				let predictions = unsafe {
					<Array1<f32>>::uninit(train_options.n_examples_per_batch).assume_init()
				};
				let metric = BinaryCrossEntropy::new();
				(predictions, metric)
			},
			|(mut predictions, mut metric), (features, labels)| {
				let slice = s![0..features.nrows()];
				let mut predictions_slice = predictions.slice_mut(slice);
				self.predict(features, predictions_slice.view_mut());
				for (prediction, label) in zip!(predictions_slice.iter(), labels.iter()) {
					metric.update(BinaryCrossEntropyInput {
						probability: *prediction,
						label: *label,
					});
				}
				(predictions, metric)
			},
		)
		.map(|(_, metric)| metric)
		.reduce(BinaryCrossEntropy::new, |mut a, b| {
			a.merge(b);
			a
		})
		.finalize()
		.unwrap()
	}

	/// Write predicted probabilities into `probabilities` for the input `features`.
	pub fn predict(&self, features: ArrayView2<f32>, mut probabilities: ArrayViewMut1<f32>) {
		probabilities.fill(self.bias);
		ndarray::linalg::general_mat_vec_mul(
			1.0,
			&features,
			&self.weights,
			1.0,
			&mut probabilities,
		);
		for probability in probabilities.iter_mut() {
			*probability = 1.0 / (probability.neg().exp() + 1.0);
		}
	}

	pub fn compute_feature_contributions(
		&self,
		features: ArrayView2<f32>,
	) -> Vec<ComputeShapValuesForExampleOutput> {
		features
			.axis_iter(Axis(0))
			.map(|features| {
				compute_shap_values_for_example(
					features.as_slice().unwrap(),
					self.bias,
					self.weights.view(),
					&self.means,
				)
			})
			.collect()
	}

	pub fn from_reader(
		binary_classifier: crate::serialize::BinaryClassifierReader,
	) -> BinaryClassifier {
		crate::serialize::deserialize_binary_classifier(binary_classifier)
	}

	pub fn to_writer(
		&self,
		writer: &mut buffalo::Writer,
	) -> buffalo::Position<crate::serialize::BinaryClassifierWriter> {
		crate::serialize::serialize_binary_classifier(self, writer)
	}

	#[must_use]
	pub fn from_bytes(&self, bytes: &[u8]) -> BinaryClassifier {
		let reader = buffalo::read::<crate::serialize::BinaryClassifierReader>(bytes);
		Self::from_reader(reader)
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		// Create the writer.
		let mut writer = buffalo::Writer::new();
		self.to_writer(&mut writer);
		writer.into_bytes()
	}
}
