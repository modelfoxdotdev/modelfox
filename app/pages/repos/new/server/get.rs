use crate::page::{Owner, Page, PageProps};
use html::html;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{redirect_to_login, service_unavailable},
	user::{authorize_user, User},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;

pub async fn get(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_props = get_app_layout_props(&context).await?;
	let owners = match user {
		User::Root => None,
		User::Normal(user) => {
			let mut owners = vec![Owner {
				value: format!("user:{}", user.id),
				title: user.email,
			}];
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
			.bind(&user.id.to_string())
			.fetch_all(&mut *db)
			.await?;
			for row in rows {
				let id: String = row.get(0);
				let title: String = row.get(1);
				owners.push(Owner {
					value: format!("organization:{}", id),
					title,
				})
			}
			Some(owners)
		}
	};
	let props = PageProps {
		app_layout_props,
		owners,
		error: None,
		owner: None,
		title: None,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
