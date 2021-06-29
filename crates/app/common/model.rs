use crate::storage::{Storage, StorageEntity};
use anyhow::Result;
use memmap::Mmap;
use tangram_id::Id;

/// Retrieves the model with the specified id.
pub async fn get_model_bytes(data_storage: &Storage, model_id: Id) -> Result<Mmap> {
	let path = data_storage.get(StorageEntity::Model, model_id).await?;
	let file = std::fs::File::open(path)?;
	let mmap = unsafe { Mmap::map(&file)? };
	Ok(mmap)
}
