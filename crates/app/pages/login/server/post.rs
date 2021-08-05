use crate::page::{Page, Stage};
use anyhow::Result;
use chrono::prelude::*;
use lettre::AsyncTransport;
use pinwheel::prelude::*;
use rand::Rng;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, service_unavailable},
	Context,
};
use tangram_id::Id;

#[derive(serde::Deserialize)]
struct Action {
	email: String,
	code: Option<String>,
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	// Read the post data.
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let Action { email, code } = match serde_urlencoded::from_bytes(&data) {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	// Upsert the user.
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
	.bind(&email)
	.execute(&mut *db)
	.await?;
	// Retrieve the user's id.
	let user_id: String = sqlx::query(
		"
			select
				id
			from users
			where
				email = $1
		",
	)
	.bind(&email)
	.fetch_one(&mut *db)
	.await?
	.get(0);
	let user_id: Id = user_id.parse()?;
	if context.options.auth_enabled() {
		if let Some(code) = code {
			// Verify the code.
			let ten_minutes_in_seconds: i32 = 10 * 60;
			let now = Utc::now().timestamp();
			let row = sqlx::query(
				"
					select
						codes.id as code_id
					from users
					join codes
					on codes.user_id = users.id
					where
						codes.used is false and
						$1 - codes.date < $2 and
						users.email = $3 and
						codes.code = $4
				",
			)
			.bind(&now)
			.bind(&ten_minutes_in_seconds)
			.bind(&email)
			.bind(&code)
			.fetch_optional(&mut db)
			.await?;
			let row = if let Some(row) = row {
				row
			} else {
				let page = Page {
					stage: Some(Stage::Code),
					error: Some("invalid code".to_owned()),
					email: Some(email),
				};
				let html = html(page);
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))?;
				return Ok(response);
			};
			let code_id: String = row.get(0);
			let code_id: Id = code_id.parse()?;
			// Delete the code.
			sqlx::query(
				"
					update codes
					set
						used = true
					where
						id = $1
				",
			)
			.bind(&code_id.to_string())
			.execute(&mut db)
			.await?;
		} else {
			// Generate a code and redirect back to the login form.
			let code: u64 = rand::thread_rng().gen_range(0..1_000_000);
			let code = format!("{:06}", code);
			let now = Utc::now().timestamp();
			let code_id = Id::generate();
			sqlx::query(
				"
					insert into codes (
						id, date, used, user_id, code
					) values (
						$1, $2, $3, $4, $5
					)
				",
			)
			.bind(&code_id.to_string())
			.bind(&now)
			.bind(false)
			.bind(&user_id.to_string())
			.bind(&code)
			.execute(&mut *db)
			.await?;
			if let Some(smtp_transport) = context.smtp_transport.clone() {
				send_code_email(smtp_transport, email.clone(), code).await?;
			}
			db.commit().await?;
			let response = http::Response::builder()
				.status(http::StatusCode::SEE_OTHER)
				.header(
					http::header::LOCATION,
					format!("/login?stage=code&email={}", email),
				)
				.body(hyper::Body::empty())?;
			return Ok(response);
		}
	}
	let token = create_token(&mut db, user_id).await?;
	db.commit().await?;
	let set_cookie = set_cookie_header_value(token, context.options.cookie_domain.as_deref());
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/")
		.header(http::header::SET_COOKIE, set_cookie)
		.body(hyper::Body::empty())?;
	Ok(response)
}

async fn create_token(db: &mut sqlx::Transaction<'_, sqlx::Any>, user_id: Id) -> Result<Id> {
	let id = Id::generate();
	let token = Id::generate();
	sqlx::query(
		"
			insert into tokens (
				id, token, user_id
			) values (
				$1, $2, $3
			)
		",
	)
	.bind(&id.to_string())
	.bind(&token.to_string())
	.bind(&user_id.to_string())
	.execute(db)
	.await?;
	Ok(token)
}

fn set_cookie_header_value(token: Id, domain: Option<&str>) -> String {
	let domain = domain.map(|domain| format!(";domain={}", domain));
	let path = Some(";path=/");
	let max_age = Some(";max-age=31536000");
	let same_site = if domain.is_some() {
		Some(";samesite=lax")
	} else {
		None
	};
	let secure = if domain.is_some() {
		Some(";secure")
	} else {
		None
	};
	format!(
		"tangram_token={}{}{}{}{}{}",
		token,
		domain.as_deref().unwrap_or(""),
		path.unwrap_or(""),
		max_age.unwrap_or(""),
		same_site.unwrap_or(""),
		secure.unwrap_or("")
	)
}

async fn send_code_email(
	smtp_transport: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
	email: String,
	code: String,
) -> Result<()> {
	let email = lettre::Message::builder()
		.from("Tangram <noreply@tangram.dev>".parse()?)
		.to(email.parse()?)
		.subject("Tangram Login Code")
		.body(format!("Your Tangram login code is {}.", code))?;
	smtp_transport.send(email).await?;
	Ok(())
}
