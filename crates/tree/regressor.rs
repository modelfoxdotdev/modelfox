use crate::{
	shap::{compute_shap_values_for_example, ComputeShapValuesForExampleOutput},
	train::{train, Task, TrainOutput},
	train_tree::TrainTree,
	Progress, TrainOptions, Tree,
};
use ndarray::prelude::*;
use num::ToPrimitive;
use rayon::prelude::*;
use tangram_table::prelude::*;
use tangram_zip::{pzip, zip};

/// `Regressor`s predict continuous target values, for example the selling price of a home.
#[derive(Clone, Debug)]
pub struct Regressor {
	/// The initial prediction of the model given no trained trees. The bias is calculated using the mean value of the target column in the training dataset.
	pub bias: f32,
	/// The trees for this model.
	pub trees: Vec<Tree>,
}

/// This struct is returned by `Regressor::train`.
#[derive(Debug)]
pub struct RegressorTrainOutput {
	/// This is the model you just trained.
	pub model: Regressor,
	/// These are the loss values for each epoch.
	pub losses: Option<Vec<f32>>,
	/// These are the importances of each feature as measured by the number of times each feature was used in a branch node.
	pub feature_importances: Option<Vec<f32>>,
}

impl Regressor {
	/// Train a regressor.
	pub fn train(
		features: TableView,
		labels: NumberTableColumnView,
		train_options: &TrainOptions,
		progress: Progress,
	) -> RegressorTrainOutput {
		let task = Task::Regression;
		let train_output = train(
			task,
			features,
			TableColumnView::Number(labels),
			train_options,
			progress,
		);
		match train_output {
			TrainOutput::Regressor(train_output) => train_output,
			_ => unreachable!(),
		}
	}

	/// Make predictions.
	pub fn predict(&self, features: ArrayView2<TableValue>, mut predictions: ArrayViewMut1<f32>) {
		predictions.fill(self.bias);
		let predictions = predictions.as_slice_mut().unwrap();
		for tree in self.trees.iter() {
			zip!(features.axis_iter(Axis(0)), predictions.iter_mut()).for_each(
				|(example, prediction)| {
					*prediction += tree.predict(example.as_slice().unwrap());
				},
			)
		}
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

	pub fn from_reader(regressor: crate::serialize::RegressorReader) -> Regressor {
		crate::serialize::deserialize_regressor(regressor)
	}

	pub fn to_writer(
		&self,
		writer: &mut buffalo::Writer,
	) -> buffalo::Position<crate::serialize::RegressorWriter> {
		crate::serialize::serialize_regressor(self, writer)
	}

	#[must_use]
	pub fn from_bytes(&self, bytes: &[u8]) -> Regressor {
		let reader = buffalo::read::<crate::serialize::RegressorReader>(bytes);
		Self::from_reader(reader)
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		// Create the writer.
		let mut writer = buffalo::Writer::new();
		self.to_writer(&mut writer);
		writer.into_bytes()
	}
}

/// This function is used by the common train function to update the logits after each round of trees is trained for regression.
pub fn update_logits(
	trees_for_round: &[TrainTree],
	features: ArrayView2<TableValue>,
	mut predictions: ArrayViewMut2<f32>,
) {
	for (prediction, features) in zip!(predictions.row_mut(0), features.axis_iter(Axis(0))) {
		for tree in trees_for_round {
			*prediction += tree.predict(features.as_slice().unwrap());
		}
	}
}

/// This function is used by the common train function to compute the loss after each tree is trained for regression.
pub fn compute_loss(predictions: ArrayView2<f32>, labels: ArrayView1<f32>) -> f32 {
	let mut loss = 0.0;
	for (label, prediction) in zip!(labels, predictions) {
		loss += 0.5 * (label - prediction) * (label - prediction)
	}
	loss / labels.len().to_f32().unwrap()
}

/// This function is used by the common train function to compute the biases for regression.
pub fn compute_biases(labels: ArrayView1<f32>) -> Array1<f32> {
	arr1(&[labels.mean().unwrap()])
}

/// This function is used by the common train function to compute the gradients and hessian after each round.
pub fn compute_gradients_and_hessians(
	// (n_examples)
	gradients: &mut [f32],
	// (n_examples)
	_hessians: &mut [f32],
	// (n_examples)
	labels: &[f32],
	// (n_examples)
	predictions: &[f32],
) {
	pzip!(gradients, labels, predictions).for_each(|(gradient, label, prediction)| {
		*gradient = prediction - label;
	});
}
