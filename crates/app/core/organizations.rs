use anyhow::Result;
use sqlx::prelude::*;
use tangram_id::Id;

pub struct GetOrganizationOutput {
	pub id: Id,
	pub name: String,
	pub members: Vec<Member>,
}

pub struct Member {
	pub id: Id,
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
				id: user_id.parse().unwrap(),
				email: row.get(1),
				is_admin: row.get(2),
			}
		})
		.collect();
	Ok(Some(GetOrganizationOutput {
		id: organization_id,
		members,
		name: organization_name,
	}))
}

pub async fn get_organization_user(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	user_id: Id,
) -> Result<Option<Member>> {
	let member_row = sqlx::query(
		"
			select
				users.id,
				users.email,
				organizations_users.is_admin
			from users
			join organizations_users
				on organizations_users.organization_id = $1
				and organizations_users.user_id = users.id
			where users.id = $2
		",
	)
	.bind(&organization_id.to_string())
	.bind(&user_id.to_string())
	.fetch_optional(&mut *db)
	.await?;
	Ok(member_row.map(|member_row| Member {
		id: user_id,
		email: member_row.get(1),
		is_admin: member_row.get(2),
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

pub async fn delete_organization(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<()> {
	sqlx::query(
		"
		delete from organizations
		where
			id = $1
	",
	)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	Ok(())
}
