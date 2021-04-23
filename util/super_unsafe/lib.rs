/*!
`SuperUnsafe` is the ultimate escape hatch to the Rust borrow checker. By wrapping a value in `SuperUnsafe`, you can simultaneously hold multiple mutable references to it, allowing you to mutate it concurrently from multiple threads.
*/

use std::cell::UnsafeCell;

pub struct SuperUnsafe<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SuperUnsafe<T> {}

impl<T> SuperUnsafe<T> {
	/// Wrap a value with `SuperUnsafe` in preparation to acquire multiple mutable references to it.
	pub fn new(value: T) -> SuperUnsafe<T> {
		SuperUnsafe(UnsafeCell::new(value))
	}

	/// Get a mutable reference to your value with absolutely no borrow checking. Make sure you know what you are doing!
	#[allow(clippy::mut_from_ref, clippy::missing_safety_doc)]
	pub unsafe fn get(&self) -> &mut T {
		&mut *self.0.get()
	}

	/// When you are done, call this function to return the value back to safety.
	pub fn into_inner(self) -> T {
		self.0.into_inner()
	}
}
