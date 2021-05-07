use std::path::PathBuf;
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
	pub cache_path: PathBuf,
}

impl Storage {
	pub async fn get_path(&self, entity: StorageEntity, id: Id) -> Result<PathBuf> {
		let path = match self {
			Storage::Local(s) => &s.path,
			Storage::S3(s) => &s.cache_path,
		};
		let entity_dir = path.join(entity.dir_name());
		let path = entity_dir.join(id.to_string());
		Ok(path)
	}

	pub async fn write(&self, entity: StorageEntity, id: Id, data: &[u8]) -> Result<()> {
		match self {
			Storage::Local(s) => {
				let entity_dir = s.path.join(entity.dir_name());
				fs::create_dir_all(&entity_dir).await?;
				let item_path = entity_dir.join(id.to_string());
				fs::write(item_path, data).await?;
				Ok(())
			}
			Storage::S3(_) => unimplemented!(),
		}
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
