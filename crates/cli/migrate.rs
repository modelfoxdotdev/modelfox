use super::app::default_database_url;
use crate::MigrateArgs;
use anyhow::Result;

pub fn migrate(args: MigrateArgs) -> Result<()> {
	let database_url = match args.database_url {
		Some(database_url) => database_url.parse()?,
		None => default_database_url(),
	};
	tangram_app::migrate(database_url)
}
