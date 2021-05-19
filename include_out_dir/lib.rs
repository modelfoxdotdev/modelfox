use sha2::Digest;
use std::{collections::HashMap, path::Path};

pub use include_out_dir_macro::include_out_dir;

pub struct IncludeOutDir(pub HashMap<&'static Path, File>);

pub struct File {
	pub data: &'static [u8],
	pub hash: String,
}

impl IncludeOutDir {
	pub fn new(map: HashMap<&'static Path, File>) -> IncludeOutDir {
		IncludeOutDir(map)
	}

	pub fn read(&self, path: &Path) -> Option<&File> {
		self.0.get(path)
	}
}

pub fn hash(data: &[u8]) -> String {
	let mut hash: sha2::Sha256 = Digest::new();
	hash.update(data);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}
