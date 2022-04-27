use modelfox_zip::zip;
use num::ToPrimitive;
use rand::random;
use std::num::NonZeroU64;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NumberStats {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f64,
	pub m2: f64,
	/// Keep a reservoir of random samples to get an estimate for quantiles.
	pub reservoir: Vec<f32>,
	pub reservoir_max_size: usize,
}

#[derive(Debug)]
pub struct NumberStatsOutput {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

impl NumberStats {
	pub fn new(value: f32) -> NumberStats {
		NumberStats {
			n: 1,
			min: value,
			max: value,
			mean: value as f64,
			m2: 0.0,
			reservoir: vec![value],
			reservoir_max_size: 100,
		}
	}

	pub fn update(&mut self, value: f32) {
		let (new_mean, new_m2) =
			modelfox_metrics::merge_mean_m2(self.n, self.mean, self.m2, 1, value as f64, 0.0);
		self.n += 1;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, value);
		self.max = f32::max(self.max, value);
		if self.reservoir.len() < self.reservoir_max_size {
			self.reservoir.push(value)
		} else {
			let index = (random::<f32>() * self.n.to_f32().unwrap())
				.floor()
				.to_usize()
				.unwrap();
			if index < self.reservoir_max_size {
				self.reservoir[index] = value;
			}
		}
	}

	pub fn merge(&mut self, other: NumberStats) {
		let (new_mean, new_m2) = modelfox_metrics::merge_mean_m2(
			self.n, self.mean, self.m2, other.n, other.mean, other.m2,
		);
		self.n += other.n;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, other.min);
		self.max = f32::max(self.max, other.max);
		self.reservoir.extend(other.reservoir);
	}

	pub fn finalize(self) -> NumberStatsOutput {
		let reservoir_len = self.reservoir.len().to_f32().unwrap();
		let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
		// Find the index of each quantile given the total number of values in the dataset.
		let quantile_indexes: Vec<usize> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).trunc().to_usize().unwrap())
			.collect();
		// This is the fractiononal part of the index used to interpolate values if the index is not an integer value.
		let quantile_fracts: Vec<f32> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).fract())
			.collect();
		let mut quantiles: Vec<f32> = vec![0.0; quantiles.len()];
		let mut samples = self.reservoir;
		samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
		for (quantile, index, fract) in zip!(
			quantiles.iter_mut(),
			quantile_indexes.iter(),
			quantile_fracts.iter(),
		) {
			let value = samples[*index];
			if *fract > 0.0 {
				let next_value = samples[index + 1];
				// Interpolate between two values.
				*quantile = value * (1.0 - fract) + next_value * fract;
			} else {
				*quantile = value;
			}
		}
		NumberStatsOutput {
			n: self.n,
			p25: quantiles[0],
			p50: quantiles[1],
			p75: quantiles[2],
			mean: self.mean.to_f32().unwrap(),
			variance: modelfox_metrics::m2_to_variance(self.m2, NonZeroU64::new(self.n).unwrap()),
			std: modelfox_metrics::m2_to_variance(self.m2, NonZeroU64::new(self.n).unwrap()).sqrt(),
			min: self.min,
			max: self.max,
		}
	}
}
