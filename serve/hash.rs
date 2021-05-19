use sha2::Digest;

pub fn hash(bytes: impl AsRef<[u8]>) -> String {
	let mut hash: sha2::Sha256 = Digest::new();
	hash.update(bytes);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}
