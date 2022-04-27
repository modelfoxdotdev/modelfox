use crate::{
	repos::add_model_version,
	storage::{BytesOrFilePath, Storage, StorageEntity},
	App,
};
use anyhow::Result;
use memmap::{Mmap, MmapMut};
use modelfox_id::Id;
use sqlx::Acquire;
use std::{
	io::{Read, Write},
	path::Path,
};

/// Retrieves the model with the specified id.
pub async fn get_model_bytes(data_storage: &Storage, model_id: Id) -> Result<Mmap> {
	let stored_model = data_storage.get(StorageEntity::Model, model_id).await?;
	let mmap = match stored_model {
		BytesOrFilePath::Path(path) => {
			let file = std::fs::File::open(path)?;
			unsafe { Mmap::map(&file)? }
		}
		BytesOrFilePath::Bytes(bytes) => {
			let mut mmap = MmapMut::map_anon(bytes.len())?;
			(&mut mmap[..]).write_all(&bytes)?;
			mmap.make_read_only()?
		}
	};
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
		let model = modelfox_model::from_bytes(&bytes)?;
		let model_id = model.id().parse().unwrap();
		add_model_version(&mut txn, self, repo_id, model_id, &bytes).await?;
		txn.commit().await?;
		Ok(model_id)
	}
}
