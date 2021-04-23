use super::mean::Mean;

/// The accuracy is the proportion of examples where predicted == label.
#[derive(Default)]
pub struct Accuracy(Mean);

impl Accuracy {
	pub fn new() -> Accuracy {
		Accuracy::default()
	}
}

impl Accuracy {
	pub fn update(&mut self, value: (usize, usize)) {
		self.0.update(if value.0 == value.1 { 1.0 } else { 0.0 })
	}

	pub fn merge(&mut self, other: Accuracy) {
		self.0.merge(other.0)
	}

	pub fn finalize(self) -> Option<f32> {
		self.0.finalize()
	}
}
