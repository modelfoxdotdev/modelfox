
// pub fn download(request: &mut http::Request<hyper::Body>) -> HandleOutput {
// 	download_inner(request).boxed()
// }

// pub async fn download_inner(
// 	request: &mut http::Request<hyper::Body>,
// ) -> Result<http::Response<hyper::Body>> {
// 	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
// 	let model_id = if let ["repos", _, "models", model_id, "download"] =
// 		path_components(&request).as_slice()
// 	{
// 		model_id.to_owned()
// 	} else {
// 		return Err(err!("unexpected path"));
// 	};
// 	let mut db = match context.database_pool.begin().await {
// 		Ok(db) => db,
// 		Err(_) => return Ok(service_unavailable()),
// 	};
// 	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
// 		Ok(user) => user,
// 		Err(_) => return Ok(redirect_to_login()),
// 	};
// 	let model_id: Id = match model_id.parse() {
// 		Ok(model_id) => model_id,
// 		Err(_) => return Ok(bad_request()),
// 	};
// 	if !authorize_user_for_model(&mut db, &user, model_id).await? {
// 		return Ok(not_found());
// 	}
// 	let bytes = get_model_bytes(&context.storage, model_id).await?;
// 	let bytes = bytes.to_owned();
// 	db.commit().await?;
// 	let response = http::Response::builder()
// 		.status(http::StatusCode::OK)
// 		.body(hyper::Body::from(bytes))
// 		.unwrap();
// 	Ok(response)
// }
