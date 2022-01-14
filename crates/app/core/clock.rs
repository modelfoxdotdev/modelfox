//! This module wraps the system clock allowing tests to mock the passage of time.

#[cfg(test)]
use std::sync::{Arc, RwLock};

use crate::App;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct Clock {
	#[cfg(test)]
	mocked_time: Arc<RwLock<OffsetDateTime>>,
}

impl Clock {
	pub fn new() -> Self {
		Self {
			#[cfg(test)]
			mocked_time: Arc::new(RwLock::new(OffsetDateTime::UNIX_EPOCH)),
		}
	}

	#[cfg(not(test))]
	pub fn now_utc(&self) -> OffsetDateTime {
		OffsetDateTime::now_utc()
	}

	#[cfg(test)]
	pub fn now_utc(&self) -> OffsetDateTime {
		let ret = if let Ok(read_guard) = self.mocked_time.read() {
			*read_guard
		} else {
			panic!("Could not read time!");
		};
		ret
	}

	#[cfg(test)]
	pub fn set_mock_time(&self, time: OffsetDateTime) {
		if let Ok(mut write_guard) = self.mocked_time.write() {
			*write_guard = time;
		} else {
			panic!("Could not acquire write guard for clock!");
		}
	}

	#[cfg(test)]
	pub fn add_mock_duration(&self, duration: time::Duration) {
		if let Ok(mut write_guard) = self.mocked_time.write() {
			*write_guard += duration;
		} else {
			panic!("Could not acquire write guard for clock!");
		}
	}
}

impl Default for Clock {
	fn default() -> Self {
		Self::new()
	}
}

impl App {
	pub fn clock(&self) -> &Clock {
		&self.state.clock
	}
}
