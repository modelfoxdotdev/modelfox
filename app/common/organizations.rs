use sqlx::prelude::*;
use tangram_error::Result;
use tangram_id::Id;

pub struct GetOrganizationOutput {
	pub id: String,
	pub name: String,
	pub members: Vec<Member>,
}

pub struct Member {
	pub id: String,
	pub email: String,
	pub is_admin: bool,
}

pub async fn get_organization(
	organization_id: Id,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Option<GetOrganizationOutput>> {
	let row = sqlx::query(
		"
			select
				organizations.name
			from organizations
				where organizations.id = $1
		",
	)
	.bind(&organization_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let organization_name: String = row.get(0);
	let user_rows = sqlx::query(
		"
			select
				users.id,
				users.email,
				organizations_users.is_admin
			from users
			join organizations_users
				on organizations_users.organization_id = $1
				and organizations_users.user_id = users.id
		",
	)
	.bind(&organization_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	let members = user_rows
		.iter()
		.map(|row| {
			let user_id: String = row.get(0);
			Member {
				id: user_id,
				email: row.get(1),
				is_admin: row.get(2),
			}
		})
		.collect();
	Ok(Some(GetOrganizationOutput {
		id: organization_id.to_string(),
		members,
		name: organization_name,
	}))
}

pub struct GetOrganizationsOutputItem {
	pub id: String,
	pub name: String,
}

pub async fn get_organizations(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user_id: Id,
) -> Result<Vec<GetOrganizationsOutputItem>> {
	let rows = sqlx::query(
		"
			select
				organizations.id,
				organizations.name
			from organizations
			join organizations_users
				on organizations_users.organization_id = organizations.id
				and organizations_users.user_id = $1
		",
	)
	.bind(&user_id.to_string())
	.fetch_all(&mut *db)
	.await?;
	Ok(rows
		.iter()
		.map(|row| {
			let id: String = row.get(0);
			let name: String = row.get(1);
			GetOrganizationsOutputItem { id, name }
		})
		.collect())
}
