use std::path::PathBuf;
use tangram_error::Result;
use tangram_id::Id;
use tokio::fs;
use url::Url;

pub enum DataStorage {
	Local(PathBuf),
	S3(Url, PathBuf),
}

#[derive(Clone, Copy)]
pub enum DataStorageEntity {
	Model,
}

impl DataStorageEntity {
	fn dir_name(&self) -> &'static str {
		match self {
			DataStorageEntity::Model => "models",
		}
	}
}

impl DataStorage {
	pub async fn get_path(&self, entity: DataStorageEntity, id: Id) -> Result<PathBuf> {
		let path = match self {
			DataStorage::Local(path) => path,
			DataStorage::S3(_url, path) => path,
		};
		let entity_dir = path.join(entity.dir_name());
		let path = entity_dir.join(id.to_string());
		Ok(path)
	}

	pub async fn write(&self, entity: DataStorageEntity, id: Id, data: &[u8]) -> Result<()> {
		match self {
			DataStorage::Local(path) => {
				let entity_dir = path.join(entity.dir_name());
				fs::create_dir_all(&entity_dir).await?;
				let item_path = entity_dir.join(id.to_string());
				fs::write(item_path, data).await?;
				Ok(())
			}
			DataStorage::S3(_, _) => unimplemented!(),
		}
	}
}
