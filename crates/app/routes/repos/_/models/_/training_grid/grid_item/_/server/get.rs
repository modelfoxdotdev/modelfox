use super::page::Page;
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_context::Context;
use tangram_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_training_grid_common::hyperparameters_for_grid_item;
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let (model_id, grid_item_id) = if let ["repos", _, "models", model_id, "training_grid", "grid_item", grid_item_id] =
		path_components(request).as_slice()
	{
		(model_id.to_owned(), grid_item_id.to_owned())
	} else {
		bail!("unexpected path");
	};

	let mut db = match app.begin_transaction().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options().auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(app.storage(), model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let grid_item_index = grid_item_id.parse::<usize>().unwrap();
	let grid_item = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			regressor
				.train_grid_item_outputs()
				.get(grid_item_index)
				.unwrap()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			binary_classifier
				.train_grid_item_outputs()
				.get(grid_item_index)
				.unwrap()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			multiclass_classifier
				.train_grid_item_outputs()
				.get(grid_item_index)
				.unwrap()
		}
	};
	let model_hyperparameters = hyperparameters_for_grid_item(&grid_item);
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::TrainingGrid).await?;
	let page = Page {
		id: model_id.to_string(),
		model_grid_item_identifier: grid_item_id.to_owned(),
		model_hyperparameters,
		model_layout_info,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	app.commit_transaction(db).await?;
	Ok(response)
}
