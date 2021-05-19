use super::node::Node;
use futures::executor::block_on;
use futures_signals::signal::{Signal, SignalExt};

pub struct SignalNode {
	pub(crate) child: Box<Node>,
}

impl SignalNode {
	pub fn new<T, S>(signal: S) -> SignalNode
	where
		T: Into<Node>,
		S: 'static + Unpin + Signal<Item = T>,
	{
		let node = block_on(signal.first().to_future()).into();
		SignalNode {
			child: Box::new(node),
		}
	}
}

impl std::fmt::Display for SignalNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.child)
	}
}
