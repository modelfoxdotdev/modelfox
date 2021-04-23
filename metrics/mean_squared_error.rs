use super::mean::Mean;

/// The mean squared error is the sum of squared differences between the predicted value and the label.
#[derive(Default)]
pub struct MeanSquaredError(Mean);

impl MeanSquaredError {
	pub fn new() -> MeanSquaredError {
		MeanSquaredError::default()
	}
}

impl MeanSquaredError {
	pub fn update(&mut self, value: (f32, f32)) {
		self.0.update((value.1 - value.0).powi(2))
	}

	pub fn merge(&mut self, other: MeanSquaredError) {
		self.0.merge(other.0)
	}

	pub fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}
