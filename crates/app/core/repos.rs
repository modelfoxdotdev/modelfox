use crate::{
	storage::{Storage, StorageEntity},
	user::NormalUser,
	App,
};
use anyhow::Result;
use chrono::prelude::*;
use sqlx::prelude::*;
use std::borrow::BorrowMut;
use tangram_id::Id;

pub struct Repo {
	pub id: String,
	pub title: String,
	pub owner_name: Option<String>,
}

pub async fn get_repo(txn: &mut sqlx::Transaction<'_, sqlx::Any>, id: Id) -> Result<Repo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.id = $1
		",
	)
	.bind(&id.to_string())
	.fetch_one(txn.borrow_mut())
	.await?;
	let id: String = row.get(0);
	let id: Id = id.parse().unwrap();
	let title = row.get(1);
	let repo = Repo {
		id: id.to_string(),
		owner_name: None,
		title,
	};
	Ok(repo)
}

pub async fn delete_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	storage: &Storage,
	repo_id: Id,
) -> Result<()> {
	let model_version_ids = get_model_version_ids(txn, repo_id).await?;
	sqlx::query(
		"
			delete from repos
			where id = $1
		",
	)
	.bind(&repo_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	for model_id in model_version_ids.into_iter() {
		storage.remove(StorageEntity::Model, model_id).await?;
	}
	Ok(())
}

pub async fn repos_for_root(txn: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<Vec<Repo>> {
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
		",
	)
	.fetch_all(txn.borrow_mut())
	.await?;
	let repos = rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let id: Id = id.parse().unwrap();
			let title = row.get(1);
			Repo {
				id: id.to_string(),
				owner_name: None,
				title,
			}
		})
		.collect();
	Ok(repos)
}

pub async fn repos_for_user(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &NormalUser,
) -> Result<Vec<Repo>> {
	let mut repos = Vec::new();
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title
			from repos
			where repos.user_id = $1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(txn.borrow_mut())
	.await?;
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let owner_name = user.email.clone();
		repos.push(Repo {
			id,
			title,
			owner_name: Some(owner_name),
		});
	}
	let rows = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				organizations.name
			from repos
			inner join organizations
				on organizations.id = repos.organization_id
			inner join organizations_users
				on organizations_users.organization_id = repos.organization_id
				and organizations_users.user_id = $1
		",
	)
	.bind(&user.id.to_string())
	.fetch_all(txn.borrow_mut())
	.await?;
	for row in rows {
		let id = row.get(0);
		let title = row.get(1);
		let owner_name = row.get(2);
		repos.push(Repo {
			id,
			title,
			owner_name,
		});
	}
	Ok(repos)
}

pub async fn create_root_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title
			) values (
				$1, $2, $3
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&Utc::now().timestamp())
	.bind(&title)
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

pub async fn create_user_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title, user_id
			) values (
				$1, $2, $3, $4
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&Utc::now().timestamp())
	.bind(&title)
	.bind(&user_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

pub async fn create_org_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	org_id: Id,
	repo_id: Id,
	title: &str,
) -> Result<()> {
	sqlx::query(
		"
			insert into repos (
				id, created_at, title, organization_id
			) values (
				$1, $2, $3, $4
			)
		",
	)
	.bind(&repo_id.to_string())
	.bind(&Utc::now().timestamp())
	.bind(&title)
	.bind(&org_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

pub async fn add_model_version(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	repo_id: Id,
	model_id: Id,
	bytes: &[u8],
) -> Result<()> {
	sqlx::query(
		"
			insert into models (
				id, created_at, repo_id
			) values (
				$1, $2, $3
			)
		",
	)
	.bind(&model_id.to_string())
	.bind(&Utc::now().timestamp())
	.bind(&repo_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	data_storage
		.set(StorageEntity::Model, model_id, bytes)
		.await?;
	Ok(())
}

pub async fn delete_model_version(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	model_id: Id,
) -> Result<()> {
	sqlx::query(
		"
			delete from models
				where id = $1
		",
	)
	.bind(&model_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	data_storage.remove(StorageEntity::Model, model_id).await?;
	Ok(())
}

pub async fn get_model_version_ids(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	repo_id: Id,
) -> Result<Vec<Id>> {
	Ok(sqlx::query(
		"
			select
				models.id
			from models
			join repos
				on models.repo_id = repos.id
			where
			repos.id = $1
		",
	)
	.bind(&repo_id.to_string())
	.fetch_all(txn.borrow_mut())
	.await?
	.iter()
	.map(|row| row.get::<String, _>(0).parse().unwrap())
	.collect())
}

impl App {
	pub async fn create_root_repo(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		title: &str,
	) -> Result<Id> {
		let mut txn = txn.begin().await?;
		let id = Id::generate();
		create_root_repo(&mut txn, id, title).await?;
		txn.commit().await?;
		Ok(id)
	}
}
