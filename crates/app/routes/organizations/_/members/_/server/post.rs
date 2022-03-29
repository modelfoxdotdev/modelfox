use anyhow::{bail, Result};
use serde::Deserialize;
use std::{borrow::BorrowMut, sync::Arc};
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	path_components,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
};
use modelfox_id::Id;

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete")]
	Delete,
	#[serde(rename = "update_member")]
	Update(MemberFields),
}

#[derive(Debug, serde::Deserialize)]
struct MemberFields {
	#[serde(default, deserialize_with = "bool_from_string")]
	is_admin: bool,
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
	D: serde::de::Deserializer<'de>,
{
	match String::deserialize(deserializer)?.as_ref() {
		"true" => Ok(true),
		"on" => Ok(true),
		other => Err(serde::de::Error::invalid_value(
			serde::de::Unexpected::Str(other),
			&"on or true",
		)),
	}
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let (organization_id, member_id) = if let ["organizations", organization_id, "members", member_id] =
		*path_components(request).as_slice()
	{
		(organization_id.to_owned(), member_id.to_owned())
	} else {
		bail!("unexpected path");
	};
	if !app.options().auth_enabled() {
		return Ok(not_found());
	}
	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	};
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let member_id: Id = match member_id.parse() {
		Ok(member_id) => member_id,
		Err(_) => return Ok(bad_request()),
	};
	let response = match action {
		Action::Delete => {
			delete_member(&mut db, organization_id, member_id).await?;
			let redirect_location = if member_id == user.id {
				"/".to_owned()
			} else {
				format!("/organizations/{}/", organization_id)
			};
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(http::header::LOCATION, redirect_location)
				.body(hyper::Body::empty())
				.unwrap()
		}
		Action::Update(member_fields) => {
			update_member(&mut db, organization_id, member_id, member_fields).await?;
			http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(
					http::header::LOCATION,
					format!("/organizations/{}/", organization_id),
				)
				.body(hyper::Body::empty())
				.unwrap()
		}
	};
	app.commit_transaction(db).await?;
	Ok(response)
}

async fn delete_member(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	member_id: Id,
) -> Result<()> {
	sqlx::query(
		"
			delete from
				organizations_users
			where
				organization_id = $1
				and user_id = $2
		",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

async fn update_member(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	member_id: Id,
	member_fields: MemberFields,
) -> Result<()> {
	dbg!(&member_fields);
	sqlx::query(
		"
			update
				organizations_users
			set is_admin = $3
			where
				organization_id = $1
				and user_id = $2
		",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.bind(&member_fields.is_admin)
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}
