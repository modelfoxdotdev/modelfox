//! This module wraps the system clock allowing tests to mock the passage of time.

#[cfg(test)]
use std::sync::{Arc, RwLock};

use crate::{App, AppState};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct Clock {
	#[cfg(test)]
	mocked_time: Arc<RwLock<OffsetDateTime>>,
}

impl Clock {
	pub fn new() -> Self {
		#[cfg(test)]
		tokio::time::pause();

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
	pub fn pause(&self) {
		tokio::time::pause();
	}

	#[cfg(test)]
	pub async fn advance(&self, duration: std::time::Duration) {
		if let Ok(mut write_guard) = self.mocked_time.write() {
			*write_guard += duration;
		} else {
			panic!("Could not acquire write guard for clock!");
		}
		tokio::time::advance(duration).await;
	}

	#[cfg(test)]
	pub fn resume(&self) {
		tokio::time::resume();
	}
}

impl Default for Clock {
	fn default() -> Self {
		Self::new()
	}
}

impl App {
	pub fn clock(&self) -> &Clock {
		self.state.clock()
	}
}

impl AppState {
	pub fn clock(&self) -> &Clock {
		&self.clock
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn test_clock() {
		let clock = Clock::new();
		// clock should begin with the Unix epoch
		assert_eq!(clock.now_utc(), OffsetDateTime::UNIX_EPOCH);

		// clock should start frozen - grab an instant, wait for some real time, no time should elapse.
		// If not frozen, a few hundred nanoseconds would pass, failing this assertion.
		let instant = tokio::time::Instant::now();
		assert_eq!(instant.elapsed(), tokio::time::Duration::from_secs(0));

		// Add some time, assert both the tokio instant and the Clock are updated properly.
		clock.advance(std::time::Duration::from_secs(30)).await;
		let plus_thirty_secs = tokio::time::Instant::now();
		assert_eq!(instant.elapsed(), tokio::time::Duration::from_secs(30));
		assert_eq!(
			plus_thirty_secs - instant,
			tokio::time::Duration::from_secs(30)
		);
		assert_eq!(
			clock.now_utc(),
			OffsetDateTime::UNIX_EPOCH
				.checked_add(time::Duration::seconds(30))
				.unwrap()
		);

		// resume, pause again, assert that some time has passed.
		let instant = tokio::time::Instant::now();
		clock.resume();
		clock.pause();
		assert!(instant.elapsed() > tokio::time::Duration::from_secs(0));
	}
}
