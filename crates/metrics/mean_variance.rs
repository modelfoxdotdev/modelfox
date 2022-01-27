use num::ToPrimitive;
use std::iter::IntoIterator;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Default)]
pub struct MeanVariance(Option<MeanVarianceInner>);

#[derive(Debug, Clone)]
struct MeanVarianceInner {
	n: NonZeroU64,
	m2: f64,
	mean: f64,
}

pub struct MeanVarianceOutput {
	pub n: u64,
	pub mean: f32,
	pub variance: f32,
}

impl MeanVariance {
	pub fn compute(input: impl IntoIterator<Item = f32>) -> MeanVarianceOutput {
		let mut mean_variance = MeanVariance::default();
		for input in input.into_iter() {
			mean_variance.update(input);
		}
		mean_variance.finalize()
	}
}

impl MeanVariance {
	pub fn update(&mut self, input: f32) {
		match &mut self.0 {
			Some(mean_variance) => {
				let (mean, m2) = merge_mean_m2(
					mean_variance.n.get(),
					mean_variance.mean,
					mean_variance.m2,
					1,
					input as f64,
					0.0,
				);
				mean_variance.n = NonZeroU64::new(mean_variance.n.get() + 1).unwrap();
				mean_variance.mean = mean;
				mean_variance.m2 = m2;
			}
			None => {
				self.0 = Some(MeanVarianceInner {
					n: NonZeroU64::new(1).unwrap(),
					mean: input as f64,
					m2: 0.0,
				})
			}
		}
	}

	pub fn merge(&mut self, other: MeanVariance) {
		match &mut self.0 {
			Some(mean_variance) => {
				if let Some(other) = other.0 {
					let (mean, m2) = merge_mean_m2(
						mean_variance.n.get(),
						mean_variance.mean,
						mean_variance.m2,
						other.n.get(),
						other.mean,
						other.m2,
					);
					mean_variance.mean = mean;
					mean_variance.m2 = m2;
					mean_variance.n =
						NonZeroU64::new(mean_variance.n.get() + other.n.get()).unwrap();
				}
			}
			None => {
				self.0 = other.0;
			}
		}
	}

	pub fn finalize(self) -> MeanVarianceOutput {
		match self.0 {
			Some(mean_variance) => MeanVarianceOutput {
				n: mean_variance.n.get(),
				variance: m2_to_variance(mean_variance.m2, mean_variance.n),
				mean: mean_variance.mean.to_f32().unwrap(),
			},
			None => MeanVarianceOutput {
				mean: 0.0,
				variance: f32::NAN,
				n: 0,
			},
		}
	}
}

/// This function combines two separate means and variances into a single mean and variance which is useful in parallel algorithms.
pub fn merge_mean_m2(
	n_a: u64,
	mean_a: f64,
	m2_a: f64,
	n_b: u64,
	mean_b: f64,
	m2_b: f64,
) -> (f64, f64) {
	let n_a = n_a.to_f64().unwrap();
	let n_b = n_b.to_f64().unwrap();
	(
		(((n_a * mean_a) + (n_b * mean_b)) / (n_a + n_b)),
		m2_a + m2_b + (mean_b - mean_a) * (mean_b - mean_a) * (n_a * n_b / (n_a + n_b)),
	)
}

/// This function computes the variance given the `m2` and `n`.
pub fn m2_to_variance(m2: f64, n: NonZeroU64) -> f32 {
	(m2 / n.get().to_f64().unwrap()).to_f32().unwrap()
}
