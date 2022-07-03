#![warn(clippy::pedantic)]

use std::sync::atomic::{AtomicBool, Ordering};

pub struct KillChip(AtomicBool);

impl KillChip {
	#[must_use] pub const fn new() -> Self {
		KillChip(AtomicBool::new(false))
	}

	pub fn activate(&self) -> bool {
		self.0.swap(true, Ordering::SeqCst)
	}

	pub fn is_activated(&self) -> bool {
		self.0.load(Ordering::SeqCst)
	}
}

impl Default for KillChip {
	fn default() -> Self {
		KillChip(AtomicBool::new(false))
	}
}