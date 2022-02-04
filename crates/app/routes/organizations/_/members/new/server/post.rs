use anyhow::{anyhow, bail, Result};
use sqlx::prelude::*;
use std::{borrow::BorrowMut, sync::Arc};
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	path_components,
	user::NormalUser,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	App,
};
use tangram_id::Id;
use url::Url;

#[derive(serde::Deserialize)]
struct Action {
	email: String,
	is_admin: Option<String>,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let organization_id = if let ["organizations", organization_id, "members", "new"] =
		*path_components(request).as_slice()
	{
		organization_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	if !app.options().auth_enabled() {
		return Ok(not_found());
	}
	let mut txn = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(request, &mut txn).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut txn, &user, organization_id).await? {
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
	let response = add_member(
		action,
		user,
		&mut txn,
		Arc::clone(&context),
		organization_id,
	)
	.await?;
	app.commit_transaction(txn).await?;
	Ok(response)
}

async fn add_member(
	action: Action,
	user: NormalUser,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: Arc<Context>,
	organization_id: Id,
) -> Result<http::Response<hyper::Body>> {
	// Create the new user.
	let user_id = Id::generate();
	sqlx::query(
		"
			insert into users (
				id, email
			) values (
				$1, $2
			)
			on conflict (email) do update set email = excluded.email
		",
	)
	.bind(&user_id.to_string())
	.bind(&action.email)
	.execute(txn.borrow_mut())
	.await?;
	let row = sqlx::query(
		"
			select id
				from users
			where email = $1
		",
	)
	.bind(&action.email)
	.fetch_one(txn.borrow_mut())
	.await?;
	let user_id: String = row.get(0);
	let user_id: Id = user_id.parse().unwrap();
	// Add the user to the organization.
	let is_admin = if let Some(is_admin) = action.is_admin {
		is_admin == "on"
	} else {
		false
	};
	sqlx::query(
		"
			insert into organizations_users
				(organization_id, user_id, is_admin)
			values
				($1, $2, $3)
			on conflict (organization_id, user_id) do nothing
		",
	)
	.bind(&organization_id.to_string())
	.bind(&user_id.to_string())
	.bind(&is_admin)
	.execute(txn.borrow_mut())
	.await?;
	// Send the new user an invitation email.
	let url = context
		.app
		.options()
		.url
		.clone()
		.ok_or_else(|| anyhow!("url option is required when smtp is enabled"))?;
	send_invitation_email(&context.app, action.email.clone(), user.email.clone(), url).await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/organizations/{}/", organization_id),
		)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}

async fn send_invitation_email(
	app: &App,
	invitee_email: String,
	inviter_email: String,
	url: Url,
) -> Result<()> {
	let mut href = url;
	href.set_path("/login");
	href.set_query(Some(&format!("email={}", invitee_email)));
	let email = lettre::Message::builder()
		.from("Tangram <noreply@tangram.dev>".parse()?)
		.to(invitee_email.parse()?)
		.subject("Tangram Invitation")
		.body(format!(
			"{} invited you to join their team on Tangram. Click the link below to login.\n\n{}",
			inviter_email, href
		))?;
	app.send_email(email).await?;
	Ok(())
}
