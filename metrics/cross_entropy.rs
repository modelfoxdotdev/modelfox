use super::mean::Mean;
use ndarray::prelude::*;
use num::clamp;
use std::num::NonZeroUsize;

/// CrossEntropy is the loss function used in multiclass classification. [Learn more](https://en.wikipedia.org/wiki/Cross_entropy#Cross-entropy_loss_function_and_logistic_regression).
#[derive(Default)]
pub struct CrossEntropy(Mean);

impl CrossEntropy {
	pub fn new() -> CrossEntropy {
		CrossEntropy::default()
	}
}

/// The input to [`CrossEntropy`].
pub struct CrossEntropyInput<'a> {
	/// (n_classes)
	pub probabilities: ArrayView1<'a, f32>,
	pub label: Option<NonZeroUsize>,
}

/// The output from [`CrossEntropy`].
pub struct CrossEntropyOutput(pub Option<f32>);

impl CrossEntropy {
	pub fn update(&mut self, value: CrossEntropyInput) {
		let label = value.label.unwrap().get() - 1;
		let mut total = 0.0;
		for (index, &probability) in value.probabilities.indexed_iter() {
			if index == label {
				total += -clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON).ln();
			}
		}
		self.0.update(total)
	}

	pub fn merge(&mut self, other: CrossEntropy) {
		self.0.merge(other.0)
	}

	pub fn finalize(self) -> CrossEntropyOutput {
		CrossEntropyOutput(self.0.finalize())
	}
}
