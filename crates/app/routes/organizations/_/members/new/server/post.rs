use anyhow::{anyhow, bail, Result};
use lettre::AsyncTransport;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	path_components,
	user::NormalUser,
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_id::Id;
use url::Url;

#[derive(serde::Deserialize)]
struct Action {
	email: String,
	is_admin: Option<String>,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let organization_id = if let ["organizations", organization_id, "members", "new"] =
		*path_components(request).as_slice()
	{
		organization_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	if !context.options.auth_enabled() {
		return Ok(not_found());
	}
	let mut db = match context.database_pool.begin().await {
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
	let response = add_member(action, user, &mut db, &context, organization_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn add_member(
	action: Action,
	user: NormalUser,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
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
	.execute(&mut *db)
	.await?;
	let row = sqlx::query(
		"
			select id
				from users
			where email = $1
		",
	)
	.bind(&action.email)
	.fetch_one(&mut *db)
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
	.execute(&mut *db)
	.await?;
	// Send the new user an invitation email.
	if let Some(smtp_transport) = context.smtp_transport.clone() {
		let url = context
			.options
			.url
			.clone()
			.ok_or_else(|| anyhow!("url option is required when smtp is enabled"))?;
		tokio::spawn(send_invitation_email(
			smtp_transport,
			action.email.clone(),
			user.email.clone(),
			url,
		));
	}
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
	smtp_transport: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
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
	smtp_transport.send(email).await?;
	Ok(())
}
