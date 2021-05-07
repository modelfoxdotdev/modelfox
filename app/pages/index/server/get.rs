use crate::page::{Page, PageProps, ReposTableProps, ReposTableRow};
use html::html;
use tangram_app_common::{
	error::{redirect_to_login, service_unavailable},
	repos::{repos_for_root, repos_for_user},
	user::{authorize_user, User},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::Result;

pub async fn get(
	context: &Context,
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
	let app_layout_props = get_app_layout_props(context).await?;
	let repos = match user {
		User::Root => repos_for_root(&mut db).await?,
		User::Normal(user) => repos_for_user(&mut db, &user).await?,
	};
	let repos_table_props = if !repos.is_empty() {
		let rows = repos
			.iter()
			.map(|repo| ReposTableRow {
				id: repo.id.clone(),
				title: repo.title.clone(),
				owner_name: repo.owner_name.clone(),
			})
			.collect();
		Some(ReposTableProps { rows })
	} else {
		None
	};
	let props = PageProps {
		app_layout_props,
		repos_table_props,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
