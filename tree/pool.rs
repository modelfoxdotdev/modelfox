use std::sync::{Arc, Mutex};

pub struct Pool<T> {
	create_item: Box<dyn Fn() -> T>,
	max_items: usize,
	state: Arc<Mutex<State<T>>>,
}

struct State<T> {
	n_items_outstanding: usize,
	available_items: Vec<T>,
}

impl<T> Pool<T> {
	pub fn new(max_items: usize, create_item: Box<dyn Fn() -> T>) -> Pool<T> {
		Pool {
			create_item,
			max_items,
			state: Arc::new(Mutex::new(State {
				n_items_outstanding: 0,
				available_items: Vec::new(),
			})),
		}
	}

	pub fn get(&self) -> Option<PoolItem<T>> {
		let mut state = self.state.lock().unwrap();
		if let Some(item) = state.available_items.pop() {
			state.n_items_outstanding += 1;
			Some(PoolItem {
				item: Some(item),
				state: self.state.clone(),
			})
		} else if state.n_items_outstanding < self.max_items {
			state.n_items_outstanding += 1;
			let item = (self.create_item)();
			Some(PoolItem {
				item: Some(item),
				state: self.state.clone(),
			})
		} else {
			None
		}
	}
}

pub struct PoolItem<T> {
	item: Option<T>,
	state: Arc<Mutex<State<T>>>,
}

impl<T> std::ops::Deref for PoolItem<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.item.as_ref().unwrap()
	}
}

impl<T> std::ops::DerefMut for PoolItem<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.item.as_mut().unwrap()
	}
}

impl<T> Drop for PoolItem<T> {
	fn drop(&mut self) {
		let mut state = self.state.lock().unwrap();
		state.available_items.push(self.item.take().unwrap());
		state.n_items_outstanding -= 1;
	}
}
