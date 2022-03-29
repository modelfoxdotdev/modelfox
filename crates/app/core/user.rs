use std::borrow::BorrowMut;

use crate::cookies::parse_cookies;
use anyhow::Result;
use sqlx::prelude::*;
use modelfox_id::Id;

pub enum User {
	Root,
	Normal(NormalUser),
}

pub struct NormalUser {
	pub id: Id,
	pub email: String,
	pub token: String,
}

pub enum AuthorizeUserError {
	CookieAndAuthorizationHeadersAbsent,
	AuthorizationNotString,
	AuthorizationInvalid,
	CookieNotString,
	CookieParseFailed,
	CookieAuthAbsent,
	TokenUnknown,
}

pub async fn authorize_user(
	request: &http::Request<hyper::Body>,
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	auth_enabled: bool,
) -> Result<Result<User, AuthorizeUserError>> {
	// When auth is disabled, everyone is authorized as the root user.
	if !auth_enabled {
		Ok(Ok(User::Root))
	} else {
		authorize_normal_user(request, db)
			.await
			.map(|r| r.map(User::Normal))
	}
}

pub async fn authorize_normal_user(
	request: &http::Request<hyper::Body>,
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<Result<NormalUser, AuthorizeUserError>> {
	let token = if let Some(authorization) = request.headers().get(http::header::AUTHORIZATION) {
		let authorization = match authorization.to_str() {
			Ok(authorization) => authorization,
			Err(_) => return Ok(Err(AuthorizeUserError::AuthorizationNotString)),
		};
		let mut components = authorization.split(' ');
		match (components.next(), components.next()) {
			(Some("Bearer"), Some(token)) => token.to_owned(),
			_ => return Ok(Err(AuthorizeUserError::AuthorizationInvalid)),
		}
	} else if let Some(cookies) = request.headers().get(http::header::COOKIE) {
		let cookies = match cookies.to_str() {
			Ok(cookies) => cookies,
			Err(_) => return Ok(Err(AuthorizeUserError::CookieNotString)),
		};
		let cookies = match parse_cookies(cookies) {
			Ok(cookies) => cookies,
			Err(_) => return Ok(Err(AuthorizeUserError::CookieParseFailed)),
		};
		match cookies.get("modelfox_token") {
			Some(&auth_cookie) => auth_cookie.to_owned(),
			None => return Ok(Err(AuthorizeUserError::CookieAuthAbsent)),
		}
	} else {
		return Ok(Err(AuthorizeUserError::CookieAndAuthorizationHeadersAbsent));
	};
	let row = sqlx::query(
		"
			select
				users.id, users.email
			from tokens
			join users
				on users.id = tokens.user_id
			where
				tokens.token = $1
		",
	)
	.bind(&token)
	.fetch_optional(txn.borrow_mut())
	.await?;
	let row = if let Some(row) = row {
		row
	} else {
		return Ok(Err(AuthorizeUserError::TokenUnknown));
	};
	let id: String = row.get(0);
	let id: Id = id.parse().unwrap();
	let email = row.get(1);
	let user = NormalUser { id, email, token };
	Ok(Ok(user))
}

pub async fn authorize_user_for_organization(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	organization_id: Id,
) -> Result<bool> {
	match user {
		User::Root => Ok(true),
		User::Normal(user) => {
			authorize_normal_user_for_organization(txn, user, organization_id).await
		}
	}
}

pub async fn authorize_normal_user_for_organization(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &NormalUser,
	organization_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
		"
			select
				count(*) > 0
			from organizations_users
			where organizations_users.user_id = $1
				and organizations_users.organization_id = $2
		",
	)
	.bind(&user.id.to_string())
	.bind(&organization_id.to_string())
	.fetch_one(txn.borrow_mut())
	.await?
	.get(0))
}

pub async fn authorize_user_for_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	repo_id: Id,
) -> Result<bool> {
	match user {
		User::Root => Ok(true),
		User::Normal(user) => authorize_normal_user_for_repo(txn, user, repo_id).await,
	}
}

pub async fn authorize_normal_user_for_repo(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &NormalUser,
	repo_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
		"
			select
				count(*) > 0
			from repos
			left join users
				on users.id = repos.user_id
			left join organizations_users
				on organizations_users.organization_id = repos.organization_id
			and
				organizations_users.user_id = $1
			where
				repos.id = $2
		",
	)
	.bind(&user.id.to_string())
	.bind(&repo_id.to_string())
	.fetch_one(txn.borrow_mut())
	.await?
	.get(0))
}

pub async fn authorize_user_for_model(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &User,
	model_id: Id,
) -> Result<bool> {
	match user {
		User::Root => Ok(true),
		User::Normal(user) => authorize_normal_user_for_model(txn, user, model_id).await,
	}
}

pub async fn authorize_normal_user_for_model(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: &NormalUser,
	model_id: Id,
) -> Result<bool> {
	Ok(sqlx::query(
		"
			select
				count(*) > 0
			from models
			join repos
				on repos.id = models.repo_id
			left join users
				on users.id = repos.user_id
			left join organizations_users on
				organizations_users.organization_id = repos.organization_id
			and
				organizations_users.user_id = $1
			where
				models.id = $2
		",
	)
	.bind(&user.id.to_string())
	.bind(&model_id.to_string())
	.fetch_one(txn.borrow_mut())
	.await?
	.get(0))
}
