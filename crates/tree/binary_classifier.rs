use crate::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train::{train, Task, TrainOutput},
	train_tree::TrainTree,
	Progress, TrainOptions, Tree,
};
use modelfox_table::prelude::*;
use modelfox_zip::{pzip, zip};
use ndarray::prelude::*;
use num::{clamp, ToPrimitive};
use rayon::prelude::*;
use std::{num::NonZeroUsize, ops::Neg};

/// `BinaryClassifier`s predict binary target values, for example whether a patient has heart disease or not.
#[derive(Clone, Debug)]
// #[buffalo(size = "dynamic")]
pub struct BinaryClassifier {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the distribution of the unique values in target column in the training dataset.
	// #[buffalo(id = 0, required)]
	pub bias: f32,
	/// The trees for this model.
	// #[buffalo(id = 1, required)]
	pub trees: Vec<Tree>,
}

/// This struct is returned by `BinaryClassifier::train`.
#[derive(Debug)]
pub struct BinaryClassifierTrainOutput {
	/// This is the model you just trained.
	pub model: BinaryClassifier,
	/// These are the loss values for each epoch.
	pub losses: Option<Vec<f32>>,
	/// These are the importances of each feature as measured by the number of times each feature was used in a branch node.
	pub feature_importances: Option<Vec<f32>>,
}

impl BinaryClassifier {
	/// Train a binary classifier.
	pub fn train(
		features: TableView,
		labels: EnumTableColumnView,
		train_options: &TrainOptions,
		progress: Progress,
	) -> BinaryClassifierTrainOutput {
		let task = Task::BinaryClassification;
		let train_output = train(
			task,
			features,
			TableColumnView::Enum(labels),
			train_options,
			progress,
		);
		match train_output {
			TrainOutput::BinaryClassifier(train_output) => train_output,
			_ => unreachable!(),
		}
	}

	/// Make predictions.
	pub fn predict(&self, features: ArrayView2<TableValue>, mut probabilities: ArrayViewMut1<f32>) {
		probabilities.fill(self.bias);
		let probabilities = probabilities.as_slice_mut().unwrap();
		for tree in self.trees.iter() {
			zip!(features.axis_iter(Axis(0)), probabilities.iter_mut()).for_each(
				|(example, logit)| {
					*logit += tree.predict(example.as_slice().unwrap());
				},
			);
		}
		probabilities.iter_mut().for_each(|probability| {
			*probability = 1.0 / (probability.neg().exp() + 1.0);
		});
	}

	/// Compute SHAP values.
	pub fn compute_feature_contributions(
		&self,
		features: ArrayView2<TableValue>,
	) -> Vec<ComputeShapValuesForExampleOutput> {
		let trees = ArrayView1::from_shape(self.trees.len(), &self.trees).unwrap();
		features
			.axis_iter(Axis(0))
			.map(|features| {
				compute_shap_values_for_example(features.as_slice().unwrap(), trees, self.bias)
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

/// This function is used by the common train function to update the logits after each tree is trained for binary classification.
pub fn update_logits(
	trees_for_round: &[TrainTree],
	binned_features: ArrayView2<TableValue>,
	mut predictions: ArrayViewMut2<f32>,
) {
	for tree in trees_for_round {
		for (prediction, features) in
			zip!(predictions.iter_mut(), binned_features.axis_iter(Axis(0)))
		{
			*prediction += tree.predict(features.as_slice().unwrap());
		}
	}
}

/// This function is used by the common train function to compute the loss after each tree is trained for binary classification.
pub fn compute_loss(logits: ArrayView2<f32>, labels: ArrayView1<Option<NonZeroUsize>>) -> f32 {
	let mut total = 0.0;
	for (label, logit) in zip!(labels.iter(), logits) {
		let label = (label.unwrap().get() - 1).to_f32().unwrap();
		let probability = 1.0 / (logit.neg().exp() + 1.0);
		let probability_clamped = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		total += -1.0 * label * probability_clamped.ln()
			+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln()
	}
	total / labels.len().to_f32().unwrap()
}

/// This function is used by the common train function to compute the biases for binary classification.
pub fn compute_biases(labels: ArrayView1<Option<NonZeroUsize>>) -> Array1<f32> {
	let pos_count = labels
		.iter()
		.map(|l| if l.unwrap().get() == 2 { 1 } else { 0 })
		.sum::<usize>();
	let neg_count = labels.len() - pos_count;
	let log_odds = (pos_count.to_f32().unwrap() / neg_count.to_f32().unwrap()).ln();
	arr1(&[log_odds])
}

/// This function is used by the common train function to compute the gradients and hessian after each round.
pub fn compute_gradients_and_hessians(
	// (n_examples)
	gradients: &mut [f32],
	// (n_examples)
	hessians: &mut [f32],
	// (n_examples)
	labels: &[Option<NonZeroUsize>],
	// (n_examples)
	predictions: &[f32],
) {
	pzip!(gradients, hessians, labels, predictions).for_each(
		|(gradient, hessian, label, prediction)| {
			let probability = clamp(
				sigmoid(*prediction),
				std::f32::EPSILON,
				1.0 - std::f32::EPSILON,
			);
			*gradient = probability - (label.unwrap().get() - 1).to_f32().unwrap();
			*hessian = probability * (1.0 - probability);
		},
	);
}

fn sigmoid(value: f32) -> f32 {
	1.0 / (value.neg().exp() + 1.0)
}
