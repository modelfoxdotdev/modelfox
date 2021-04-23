use num::ToPrimitive;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct Timing {
	pub choose_best_split_not_root: TimingDuration,
	pub choose_best_split_root: TimingDuration,
	pub compute_binned_features: TimingDuration,
	pub compute_binning_instructions: TimingDuration,
	pub compute_gradients_and_hessians: TimingDuration,
	pub rearrange_examples_index: TimingDuration,
	pub sum_gradients_and_hessians_root: TimingDuration,
	pub update_predictions: TimingDuration,
}

pub struct TimingDuration(AtomicU64);

impl Timing {
	pub fn new() -> Timing {
		Timing {
			compute_binning_instructions: TimingDuration::new(),
			compute_binned_features: TimingDuration::new(),
			compute_gradients_and_hessians: TimingDuration::new(),
			sum_gradients_and_hessians_root: TimingDuration::new(),
			rearrange_examples_index: TimingDuration::new(),
			choose_best_split_not_root: TimingDuration::new(),
			choose_best_split_root: TimingDuration::new(),
			update_predictions: TimingDuration::new(),
		}
	}
}

impl TimingDuration {
	pub fn new() -> TimingDuration {
		TimingDuration(AtomicU64::new(0))
	}
	pub fn get(&self) -> Duration {
		Duration::from_nanos(self.0.load(Ordering::Relaxed))
	}
	pub fn inc(&self, value: Duration) -> u64 {
		self.0
			.fetch_add(value.as_nanos().to_u64().unwrap(), Ordering::Relaxed)
	}
}

impl std::fmt::Debug for TimingDuration {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.get())
	}
}
