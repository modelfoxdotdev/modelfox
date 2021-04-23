use crate::data_storage::{DataStorage, DataStorageEntity};
use tangram_error::Result;
use tangram_id::Id;

/// Retrieves the model with the specified id.
pub async fn get_model_bytes(data_storage: &DataStorage, model_id: Id) -> Result<memmap::Mmap> {
	let path = data_storage
		.get_path(DataStorageEntity::Model, model_id)
		.await?;
	let file = std::fs::File::open(path)?;
	let mmap = unsafe { memmap::Mmap::map(&file)? };
	Ok(mmap)
}
