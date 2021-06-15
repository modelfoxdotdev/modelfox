use super::mean::Mean;
use num::clamp;
use std::num::NonZeroUsize;

/// BinaryCrossEntropy is the loss function used in binary classification. [Learn more](https://en.wikipedia.org/wiki/Cross_entropy#Cross-entropy_loss_function_and_logistic_regression).
#[derive(Debug, Default)]
pub struct BinaryCrossEntropy(Mean);

impl BinaryCrossEntropy {
	pub fn new() -> BinaryCrossEntropy {
		BinaryCrossEntropy::default()
	}
}

/// The input to [`BinaryCrossEntropy`].
pub struct BinaryCrossEntropyInput {
	pub probability: f32,
	pub label: Option<NonZeroUsize>,
}

impl BinaryCrossEntropy {
	pub fn update(&mut self, value: BinaryCrossEntropyInput) {
		let BinaryCrossEntropyInput { probability, label } = value;
		let label = match label.map(|l| l.get()) {
			Some(1) => 0.0,
			Some(2) => 1.0,
			_ => unreachable!(),
		};
		// Binary cross entropy is undefined when the probability = 0 or probability = 1, so we clamp the probability to be between (std::f32::EPSILON, 1.0-std::f32::EPSILON).
		let probability_clamped = clamp(probability, std::f32::EPSILON, 1.0 - std::f32::EPSILON);
		let binary_cross_entropy = -1.0 * label * probability_clamped.ln()
			+ -1.0 * (1.0 - label) * (1.0 - probability_clamped).ln();
		self.0.update(binary_cross_entropy);
	}

	pub fn merge(&mut self, other: BinaryCrossEntropy) {
		self.0.merge(other.0)
	}

	pub fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}
