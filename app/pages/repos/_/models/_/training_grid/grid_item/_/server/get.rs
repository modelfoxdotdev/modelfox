use super::page::{Page, PageProps};
use html::html;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{get_model_layout_props, ModelNavItem};
use tangram_app_training_grid_common::hyperparameters_for_grid_item;
use tangram_error::Result;
use tangram_id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	grid_item_identifier: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled()).await? {
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
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let grid_item_index = grid_item_identifier.parse::<usize>().unwrap();
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
	let model_layout_props =
		get_model_layout_props(&mut db, context, model_id, ModelNavItem::TrainingGrid).await?;
	let props = PageProps {
		id: model_id.to_string(),
		model_grid_item_identifier: grid_item_identifier.to_owned(),
		model_hyperparameters,
		model_layout_props,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
