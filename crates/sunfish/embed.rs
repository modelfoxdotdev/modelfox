use std::{collections::HashMap, path::Path};

#[derive(Debug)]
pub struct EmbeddedDirectory(pub HashMap<&'static Path, EmbeddedFile>);

#[derive(Debug)]
pub struct EmbeddedFile {
	pub data: &'static [u8],
	pub hash: String,
}

impl EmbeddedDirectory {
	pub fn new(map: HashMap<&'static Path, EmbeddedFile>) -> EmbeddedDirectory {
		EmbeddedDirectory(map)
	}

	pub fn read(&self, path: &Path) -> Option<&EmbeddedFile> {
		self.0.get(path)
	}
}
