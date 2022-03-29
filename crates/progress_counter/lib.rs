use std::sync::{
	atomic::{AtomicU64, Ordering},
	Arc,
};

/**
A `ProgressCounter` is used to efficiently track the progress of a task occurring across multiple threads.

Imagine you have the following code:

```
use rayon::prelude::*;

let mut v: Vec<usize> = (0..1_000).collect();
v.par_iter_mut().for_each(|v| { *v += 1; });
```

Now you want to track the progress of this loop. You can use `Arc<Mutex<T>>` like so:

```
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

let mut v: Vec<usize> = (0..1_000).collect();
let progress_counter = Arc::new(Mutex::new(0));
v.par_iter_mut().for_each(|v| {
  *v += 1;
  *progress_counter.lock().unwrap() += 1;
});
```

However, because the work done in each loop iteration is small, a large portion of time will be spent waiting on the mutex. A better choice in this case is to use [atomics](https://doc.rust-lang.org/stable/std/sync/atomic/index.html). `ProgressCounter` is a convenient wrapper around atomics for use in tracking progress. This example will now run much faster:

```
use rayon::prelude::*;
use modelfox_progress_counter::ProgressCounter;

let mut v: Vec<usize> = (0..1_000).collect();
let progress_counter = ProgressCounter::new(v.len() as u64);
v.par_iter_mut().for_each(|v| {
  *v += 1;
  progress_counter.inc(1);
});
```
*/
#[derive(Clone, Debug)]
pub struct ProgressCounter {
	current: Arc<AtomicU64>,
	total: u64,
}

impl ProgressCounter {
	/// Create a new `ProgressCounter` that will count from 0 up to the specified `total`.
	pub fn new(total: u64) -> ProgressCounter {
		ProgressCounter {
			current: Arc::new(AtomicU64::new(0)),
			total,
		}
	}

	/// Retrieve the total value this `ProgressCounter` counts up to.
	pub fn total(&self) -> u64 {
		self.total
	}

	/// Retrieve the current progress value.
	pub fn get(&self) -> u64 {
		self.current.load(Ordering::Relaxed)
	}

	/// Set the current progress value.
	pub fn set(&self, value: u64) {
		self.current.store(value, Ordering::Relaxed);
	}

	/// Increment the progress value by `amount`.
	pub fn inc(&self, amount: u64) {
		self.current.fetch_add(amount, Ordering::Relaxed);
	}
}
