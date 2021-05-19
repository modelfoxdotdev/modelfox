use super::node::Node;
use futures::{executor::block_on, stream::StreamExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};

pub struct SignalVecNode {
	pub(crate) children: Vec<Node>,
}

impl SignalVecNode {
	pub fn new<T, S>(signal_vec: S) -> SignalVecNode
	where
		T: Into<Node>,
		S: 'static + Unpin + SignalVec<Item = T>,
	{
		let mut children = Vec::new();
		block_on(
			signal_vec
				.map(|child| child.into())
				.to_stream()
				.map(|diff| {
					diff.apply_to_vec(&mut children);
				})
				.next(),
		);
		SignalVecNode { children }
	}
}

impl std::fmt::Display for SignalVecNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for child in self.children.iter() {
			write!(f, "{}", child)?;
		}
		Ok(())
	}
}
