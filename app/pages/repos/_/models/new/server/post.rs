use crate::page::{Page, PageProps};
use html::html;
use multer::Multipart;
use std::sync::Arc;
use tangram_app_common::{
	error::{not_found, redirect_to_login, service_unavailable},
	path_components,
	repos::add_model_version,
	user::{authorize_user, authorize_user_for_repo},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_props;
use tangram_error::{err, Result};
use tangram_id::Id;

pub async fn post(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let repo_id = if let &["repos", repo_id, "models", "new"] = path_components(&request).as_slice()
	{
		repo_id.to_owned()
	} else {
		return Err(err!("unexpected path"));
	};
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let repo_id: Id = match repo_id.parse() {
		Ok(repo_id) => repo_id,
		Err(_) => return Ok(not_found()),
	};
	if !authorize_user_for_repo(&mut db, &user, repo_id).await? {
		return Ok(not_found());
	}
	let app_layout_props = get_app_layout_props(&context).await?;
	let boundary = match request
		.headers()
		.get(http::header::CONTENT_TYPE)
		.and_then(|ct| ct.to_str().ok())
		.and_then(|ct| multer::parse_boundary(ct).ok())
	{
		Some(boundary) => boundary,
		None => {
			let props = PageProps {
				app_layout_props,
				error: Some(format!(
					"Failed to parse request body.\n{}:{}",
					file!(),
					line!()
				)),
			};
			let html = html!(<Page {props} />).render_to_string();
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let mut file: Option<Vec<u8>> = None;
	let mut multipart = Multipart::new(request.into_body(), boundary);
	while let Some(field) = multipart.next_field().await? {
		let name = match field.name() {
			Some(name) => name.to_owned(),
			None => {
				let props = PageProps {
					app_layout_props,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
				};
				let html = html!(<Page {props} />).render_to_string();
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		};
		let field_data = field.bytes().await?.to_vec();
		match name.as_str() {
			"file" => file = Some(field_data),
			_ => {
				let props = PageProps {
					app_layout_props,
					error: Some(format!(
						"Failed to parse request body.\n{}:{}",
						file!(),
						line!()
					)),
				};
				let html = html!(<Page {props} />).render_to_string();
				let response = http::Response::builder()
					.status(http::StatusCode::BAD_REQUEST)
					.body(hyper::Body::from(html))
					.unwrap();
				return Ok(response);
			}
		}
	}
	let bytes = match file {
		Some(file) => file,
		None => {
			let props = PageProps {
				app_layout_props,
				error: Some("A file is required.".to_owned()),
			};
			let html = html!(<Page {props} />).render_to_string();
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let model = match tangram_model::from_bytes(&bytes) {
		Ok(model) => model,
		Err(_) => {
			let props = PageProps {
				app_layout_props,
				error: Some("Invalid tangram model file.".to_owned()),
			};
			let html = html!(<Page {props} />).render_to_string();
			let response = http::Response::builder()
				.status(http::StatusCode::BAD_REQUEST)
				.body(hyper::Body::from(html))
				.unwrap();
			return Ok(response);
		}
	};
	let result = add_model_version(
		&mut db,
		&context.storage,
		repo_id,
		model.id().parse().unwrap(),
		&bytes,
	)
	.await;
	if result.is_err() {
		let props = PageProps {
			app_layout_props,
			error: Some("There was an error uploading your model.".to_owned()),
		};
		let html = html!(<Page {props} />).render_to_string();
		let response = http::Response::builder()
			.status(http::StatusCode::BAD_REQUEST)
			.body(hyper::Body::from(html))
			.unwrap();
		return Ok(response);
	};
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(
			http::header::LOCATION,
			format!("/repos/{}/models/{}/", repo_id, model.id()),
		)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
