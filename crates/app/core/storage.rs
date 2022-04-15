use crate::App;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use modelfox_id::Id;
use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, RwLock},
};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Storage {
	InMemory(InMemoryStorage),
	Local(LocalStorage),
	S3(S3Storage),
}

#[derive(Debug)]
pub struct InMemoryStorage {
	storage: Arc<RwLock<HashMap<Id, Bytes>>>,
}

#[derive(Debug)]
pub struct LocalStorage {
	pub path: PathBuf,
}

#[derive(Debug)]
pub struct S3Storage {
	bucket: s3::Bucket,
	cache_path: PathBuf,
}

pub enum BytesOrFilePath {
	Bytes(Bytes),
	Path(PathBuf),
}

impl From<Bytes> for BytesOrFilePath {
	fn from(bytes: Bytes) -> Self {
		BytesOrFilePath::Bytes(bytes)
	}
}

impl From<PathBuf> for BytesOrFilePath {
	fn from(path_buf: PathBuf) -> Self {
		BytesOrFilePath::Path(path_buf)
	}
}

impl TryFrom<BytesOrFilePath> for Bytes {
	type Error = anyhow::Error;
	fn try_from(value: BytesOrFilePath) -> Result<Self, Self::Error> {
		match value {
			BytesOrFilePath::Bytes(bytes) => Ok(bytes),
			BytesOrFilePath::Path(_) => {
				Err(anyhow!("Attempt to extract PathBuf from Bytes variant"))
			}
		}
	}
}

impl TryFrom<BytesOrFilePath> for PathBuf {
	type Error = anyhow::Error;
	fn try_from(value: BytesOrFilePath) -> Result<Self, Self::Error> {
		match value {
			BytesOrFilePath::Bytes(_) => {
				Err(anyhow!("Attempt to extract bytes from FilePath variant"))
			}
			BytesOrFilePath::Path(path_buf) => Ok(path_buf),
		}
	}
}

impl Storage {
	pub async fn get(&self, entity: StorageEntity, id: Id) -> Result<BytesOrFilePath> {
		match self {
			Storage::InMemory(s) => s.get(entity, id).await,
			Storage::Local(s) => s.get(entity, id).await,
			Storage::S3(s) => s.get(entity, id).await,
		}
	}

	pub async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		match self {
			Storage::InMemory(s) => s.set(entity, id, data).await,
			Storage::Local(s) => s.set(entity, id, data).await,
			Storage::S3(s) => s.set(entity, id, data).await,
		}
	}

	pub async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		match self {
			Storage::InMemory(s) => s.remove(entity, id).await,
			Storage::Local(s) => s.remove(entity, id).await,
			Storage::S3(s) => s.remove(entity, id).await,
		}
	}
}

impl InMemoryStorage {
	pub fn new() -> Self {
		Self {
			storage: Arc::new(RwLock::new(HashMap::new())),
		}
	}
}

impl Default for InMemoryStorage {
	fn default() -> Self {
		Self::new()
	}
}

impl InMemoryStorage {
	async fn get(&self, _entity: StorageEntity, id: Id) -> Result<BytesOrFilePath> {
		let storage = Arc::clone(&self.storage);
		let ret = if let Ok(read_guard) = storage.read() {
			if let Some(bytes) = (*read_guard).get(&id) {
				Ok(BytesOrFilePath::from(bytes.clone()))
			} else {
				Err(anyhow!("No such ID in storage"))
			}
		} else {
			Err(anyhow!("Could not access in-memory storage"))
		};
		ret
	}

	async fn set(&self, _entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		let storage = Arc::clone(&self.storage);
		if let Ok(mut write_guard) = storage.write() {
			(*write_guard).insert(id, Bytes::from(data.to_owned()));
		}
		Ok(())
	}

	async fn remove(&self, _entity: StorageEntity, id: Id) -> Result<()> {
		let storage = Arc::clone(&self.storage);
		if let Ok(mut write_guard) = storage.write() {
			(*write_guard).remove(&id);
		}
		Ok(())
	}
}

impl LocalStorage {
	async fn get(&self, entity: StorageEntity, id: Id) -> Result<BytesOrFilePath> {
		let entity_path = self.path.join(entity.dir_name());
		let path = entity_path.join(id.to_string());
		Ok(BytesOrFilePath::from(path))
	}

	async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		let entity_path = self.path.join(entity.dir_name());
		tokio::fs::create_dir_all(&entity_path).await?;
		let item_path = entity_path.join(id.to_string());
		tokio::fs::write(item_path, data).await?;
		Ok(())
	}

	async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		let entity_path = self.path.join(entity.dir_name());
		let item_path = entity_path.join(id.to_string());
		tokio::fs::remove_file(item_path).await?;
		Ok(())
	}
}

impl S3Storage {
	pub fn new(
		access_key: String,
		secret_key: String,
		endpoint: String,
		bucket: String,
		region: String,
		cache_path: PathBuf,
	) -> Result<S3Storage> {
		let credentials =
			s3::creds::Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;
		let bucket = s3::Bucket::new(
			&bucket,
			s3::Region::Custom { region, endpoint },
			credentials,
		)?;
		Ok(S3Storage { bucket, cache_path })
	}
}

impl S3Storage {
	async fn get(&self, entity: StorageEntity, id: Id) -> Result<BytesOrFilePath> {
		// Attempt to retrieve the item from the cache.
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		let item_cache_path = entity_cache_path.join(id.to_string());
		if tokio::fs::metadata(&item_cache_path).await.is_ok() {
			return Ok(BytesOrFilePath::from(item_cache_path));
		}
		// Retrieve the item from s3 and cache it.
		let (data, _) = self.bucket.get_object(&key_for_item(entity, id)).await?;
		// Add the item to the cache.
		tokio::fs::create_dir_all(&entity_cache_path).await?;
		tokio::fs::write(&item_cache_path, data).await?;
		Ok(BytesOrFilePath::from(item_cache_path))
	}

	async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		tokio::fs::create_dir_all(&entity_cache_path).await?;
		let item_cache_path = entity_cache_path.join(id.to_string());
		// Upload the item to s3.
		self.bucket
			.put_object(&key_for_item(entity, id), data)
			.await?;
		// Add the item to the cache.
		tokio::fs::write(item_cache_path, data).await?;
		Ok(())
	}

	async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		// Remove the item from the cache if it exists.
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		let item_cache_path = entity_cache_path.join(id.to_string());
		if tokio::fs::metadata(&item_cache_path).await.is_ok() {
			tokio::fs::remove_file(item_cache_path).await?;
		}
		// Remove the item from s3.
		self.bucket.delete_object(&key_for_item(entity, id)).await?;
		Ok(())
	}
}

fn key_for_item(entity: StorageEntity, id: Id) -> String {
	format!("{}/{}", entity.dir_name(), id)
}

#[derive(Clone, Copy)]
pub enum StorageEntity {
	Model,
}

impl StorageEntity {
	fn dir_name(&self) -> &'static str {
		match self {
			StorageEntity::Model => "models",
		}
	}
}

impl App {
	pub fn storage(&self) -> &Storage {
		&self.state.storage
	}
}
