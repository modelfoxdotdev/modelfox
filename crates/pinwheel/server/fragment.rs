use super::Node;

pub fn fragment() -> Fragment {
	Fragment::new()
}

pub struct Fragment {
	children: Vec<Node>,
}

impl Fragment {
	pub fn new() -> Fragment {
		Fragment {
			children: Vec::new(),
		}
	}

	pub fn child<T>(mut self, child: T) -> Fragment
	where
		T: Into<Node>,
	{
		let child = child.into();
		self.children.push(child);
		self
	}
}

impl Default for Fragment {
	fn default() -> Self {
		Self::new()
	}
}

impl std::fmt::Display for Fragment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for child in self.children.iter() {
			write!(f, "{}", child)?;
		}
		Ok(())
	}
}
