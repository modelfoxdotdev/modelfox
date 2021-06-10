use crate::page::{Page, ReposTable, ReposTableRow};
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{redirect_to_login, service_unavailable},
	repos::{repos_for_root, repos_for_user},
	user::{authorize_user, User},
	Context,
};
use tangram_app_layouts::app_layout::app_layout_info;
use tangram_error::Result;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_info = app_layout_info(&context).await?;
	let repos = match user {
		User::Root => repos_for_root(&mut db).await?,
		User::Normal(user) => repos_for_user(&mut db, &user).await?,
	};
	let repos_table = if !repos.is_empty() {
		let rows = repos
			.iter()
			.map(|repo| ReposTableRow {
				id: repo.id.clone(),
				title: repo.title.clone(),
				owner_name: repo.owner_name.clone(),
			})
			.collect();
		Some(ReposTable { rows })
	} else {
		None
	};
	let page = Page {
		app_layout_info,
		repos_table,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
