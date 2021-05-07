use std::path::{Path, PathBuf};
use tangram_error::Result;
use tangram_id::Id;
use tokio::fs;

pub enum Storage {
	Local(LocalStorage),
	S3(S3Storage),
}

pub struct LocalStorage {
	pub path: PathBuf,
}

pub struct S3Storage {
	pub access_key: String,
	pub secret_key: String,
	pub endpoint: String,
	pub bucket: String,
	pub region: String,
	pub cache_path: PathBuf,
}

impl Storage {
	pub async fn get(&self, entity: StorageEntity, id: Id) -> Result<PathBuf> {
		match self {
			Storage::Local(s) => s.get(entity, id).await,
			Storage::S3(s) => s.get(entity, id).await,
		}
	}

	pub async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		match self {
			Storage::Local(s) => s.set(entity, id, data).await,
			Storage::S3(s) => s.set(entity, id, data).await,
		}
	}

	pub async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		match self {
			Storage::Local(s) => s.remove(entity, id).await,
			Storage::S3(s) => s.remove(entity, id).await,
		}
	}
}

impl LocalStorage {
	pub async fn get(&self, entity: StorageEntity, id: Id) -> Result<PathBuf> {
		let entity_path = self.path.join(entity.dir_name());
		let path = entity_path.join(id.to_string());
		Ok(path)
	}

	pub async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		let entity_path = self.path.join(entity.dir_name());
		fs::create_dir_all(&entity_path).await?;
		let item_path = entity_path.join(id.to_string());
		fs::write(item_path, data).await?;
		Ok(())
	}

	pub async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		let entity_path = self.path.join(entity.dir_name());
		let item_path = entity_path.join(id.to_string());
		fs::remove_file(item_path).await?;
		Ok(())
	}
}

impl S3Storage {
	pub async fn get(&self, entity: StorageEntity, id: Id) -> Result<PathBuf> {
		// Attempt to retrieve the item from the cache.
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		let item_cache_path = entity_cache_path.join(id.to_string());
		if fs::metadata(&item_cache_path).await.is_ok() {
			return Ok(item_cache_path);
		}
		// Retrieve the item from s3 and cache it.
		let data: Vec<u8> = todo!();
		// Add the item to the cache.
		fs::create_dir_all(&entity_cache_path).await?;
		fs::write(&item_cache_path, data).await?;
		Ok(item_cache_path)
	}

	pub async fn set(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		fs::create_dir_all(&entity_cache_path).await?;
		let item_cache_path = entity_cache_path.join(id.to_string());
		// Upload the item to s3.
		todo!();
		// Add the item to the cache.
		fs::write(item_cache_path, data).await?;
		Ok(())
	}

	pub async fn remove(&self, entity: StorageEntity, id: Id) -> Result<()> {
		// Remove the item from the cache if it exists.
		let entity_cache_path = self.cache_path.join(entity.dir_name());
		let item_cache_path = entity_cache_path.join(id.to_string());
		if fs::metadata(&item_cache_path).await.is_ok() {
			fs::remove_file(item_cache_path).await?;
		}
		// Remove the item from s3.
		todo!();
		Ok(())
	}
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
