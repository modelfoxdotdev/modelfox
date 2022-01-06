use crate::{
	repos::add_model_version,
	storage::{Storage, StorageEntity},
	App,
};
use anyhow::Result;
use memmap::Mmap;
use sqlx::Acquire;
use std::{io::Read, path::Path};
use tangram_id::Id;

/// Retrieves the model with the specified id.
pub async fn get_model_bytes(data_storage: &Storage, model_id: Id) -> Result<Mmap> {
	let path = data_storage.get(StorageEntity::Model, model_id).await?;
	let file = std::fs::File::open(path)?;
	let mmap = unsafe { Mmap::map(&file)? };
	Ok(mmap)
}

impl App {
	pub async fn add_model_to_repo(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		repo_id: Id,
		model_path: impl AsRef<Path>,
	) -> Result<Id> {
		let mut txn = txn.begin().await?;
		let model_path = model_path.as_ref();
		let mut bytes = Vec::new();
		let mut f = std::fs::File::open(model_path)?;
		f.read_to_end(&mut bytes)?;
		let model = tangram_model::from_bytes(&bytes)?;
		let model_id = model.id().parse().unwrap();
		add_model_version(&mut txn, &self.state.storage, repo_id, model_id, &bytes).await?;
		txn.commit().await?;
		Ok(model_id)
	}
}
