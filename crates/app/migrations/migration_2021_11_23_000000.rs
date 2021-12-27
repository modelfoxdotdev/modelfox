use anyhow::Result;
use sqlx::prelude::*;

pub async fn migrate(db: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<()> {
	db.execute(include_str!("./migration_2021_11_23_000000.sql"))
		.await?;
	Ok(())
}
