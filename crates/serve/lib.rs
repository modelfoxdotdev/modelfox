use std::{borrow::Cow, collections::BTreeMap, path::Path};

pub mod build;
pub mod dir;
pub mod export;
mod hash;
pub mod request_id;

pub use crate::hash::hash;

pub fn asset_path(path: &Path) -> String {
	let extension = path.extension().map(|e| e.to_str().unwrap()).unwrap();
	let hash = hash(&path.to_str().unwrap().as_bytes());
	format!("/assets/{}.{}", hash, extension)
}

pub fn client_hash(crate_name: &'static str) -> String {
	hash(crate_name.as_bytes())
}

pub type RouteMap = BTreeMap<Cow<'static, str>, Box<dyn 'static + Send + Sync + Fn() -> String>>;
