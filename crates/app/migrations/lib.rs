use anyhow::{bail, Result};
use futures::prelude::future::*;
use once_cell::sync::Lazy;
use sqlx::prelude::*;
use std::collections::BTreeMap;
use tangram_zip::zip;

mod migration_2020_01_01_000000;
mod migration_2020_04_19_000000;
mod migration_2021_11_23_000000;

#[rustfmt::skip]
type Migration = &'static (dyn Sync + for<'a> Fn(&'a mut sqlx::Transaction<sqlx::Any>) -> BoxFuture<'a, Result<()>>);

type MigrationMap = BTreeMap<&'static str, Migration>;

static MIGRATIONS: Lazy<MigrationMap> = Lazy::new(|| {
	let mut migrations: MigrationMap = BTreeMap::new();
	migrations.insert("2020_01_01_000000", &|db| {
		migration_2020_01_01_000000::migrate(db).boxed()
	});
	migrations.insert("2020_04_19_000000", &|db| {
		migration_2020_04_19_000000::migrate(db).boxed()
	});
	migrations.insert("2021_11_23_000000", &|db| {
		migration_2021_11_23_000000::migrate(db).boxed()
	});
	migrations
});

/// Determine if any migrations have been run on the database yet.
pub async fn empty(db: &sqlx::AnyPool) -> Result<bool> {
	create_migrations_table_if_necessary(db).await?;
	let empty = sqlx::query("select count(*) = 0 from _migrations")
		.fetch_one(db)
		.await?
		.get(0);
	Ok(empty)
}

/// Verify that the database has run all migrations.
pub async fn verify(db: &sqlx::AnyPool) -> Result<()> {
	create_migrations_table_if_necessary(db).await?;
	let migration_rows = sqlx::query("select name from _migrations order by name")
		.fetch_all(db)
		.await?;
	let migrations_consistent =
		zip!(migration_rows.iter(), MIGRATIONS.keys()).all(|(migration_row, migration_name)| {
			migration_row.get::<String, usize>(0) == *migration_name
		});
	if !migrations_consistent {
		bail!(
			"There was a mismatch between the migrations your database has run and the migrations this version of tangram expects. This should not happen unless you are hacking on tangram. Please contact us at help@tangram.dev."
		);
	}
	if migration_rows.len() > MIGRATIONS.len() {
		bail!(
			"Your database has run migrations from a newer version of tangram. Please update to the latest version of tangram."
		);
	}
	if migration_rows.len() < MIGRATIONS.len() {
		bail!("Please run `tangram migrate` to update your database to the latest schema.");
	}
	Ok(())
}

/// Run all outstanding migrations.
pub async fn run(db: &sqlx::AnyPool) -> Result<()> {
	create_migrations_table_if_necessary(db).await?;
	let mut db = db.begin().await?;
	let migration_rows = sqlx::query("select name from _migrations order by name")
		.fetch_all(&mut db)
		.await?;
	let migrations_consistent =
		zip!(migration_rows.iter(), MIGRATIONS.keys()).all(|(migration_row, migration_name)| {
			migration_row.get::<String, usize>(0) == *migration_name
		});
	if !migrations_consistent {
		bail!("Database migration consistency error. Please contact us at help@tangram.dev.");
	}
	if migration_rows.len() > MIGRATIONS.len() {
		bail!("Your database has run migrations from a newer version of tangram.");
	}
	// Apply each outstanding migration in a transaction.
	for (migration_name, migration) in MIGRATIONS.iter().skip(migration_rows.len()) {
		let mut db = db.begin().await?;
		migration(&mut db).await?;
		sqlx::query(
			"
				insert into _migrations (name) values ($1)
			",
		)
		.bind(migration_name)
		.execute(&mut db)
		.await?;
		db.commit().await?;
	}
	db.commit().await?;
	Ok(())
}

/// Create the _migrations table if necessary.
async fn create_migrations_table_if_necessary(db: &sqlx::AnyPool) -> Result<()> {
	sqlx::query(
		"
			create table if not exists _migrations (
				name text primary key
			)
		",
	)
	.execute(db)
	.await?;
	Ok(())
}
