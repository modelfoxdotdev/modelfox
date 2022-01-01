use crate::page::{ConfusionMatrixSection, Page, PrecisionRecallSection};
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
use tangram_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let model_id = if let ["repos", _, "models", model_id, "training_metrics", "class_metrics"] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
	} else {
		bail!("unexpected path");
	};
	#[derive(serde::Deserialize, Default)]
	struct SearchParams {
		class: Option<String>,
	}
	let search_params: Option<SearchParams> = if let Some(query) = request.uri().query() {
		Some(serde_urlencoded::from_str(query)?)
	} else {
		None
	};
	let mut db = match app.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, app.options.auth_enabled()).await? {
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
	let bytes = get_model_bytes(&app.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let multiclass_classifier = match model.inner() {
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read()
		}
		_ => return Ok(bad_request()),
	};
	let class = search_params.and_then(|s| s.class);
	let classes: Vec<String> = multiclass_classifier
		.classes()
		.iter()
		.map(ToOwned::to_owned)
		.collect();
	let class_index = if let Some(class) = &class {
		classes.iter().position(|c| c == class).unwrap()
	} else {
		0
	};
	let class = class.unwrap_or_else(|| classes.get(class_index).unwrap().clone());
	let test_metrics = multiclass_classifier.test_metrics();
	let class_metrics = test_metrics.class_metrics().get(class_index).unwrap();
	let precision = class_metrics.precision();
	let recall = class_metrics.recall();
	let f1_score = class_metrics.f1_score();
	let true_negatives = class_metrics.true_negatives();
	let true_positives = class_metrics.true_positives();
	let false_negatives = class_metrics.false_negatives();
	let false_positives = class_metrics.false_positives();
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::TrainingMetrics).await?;
	let precision_recall_section = PrecisionRecallSection {
		f1_score,
		precision,
		recall,
		class: class.clone(),
	};
	let confusion_matrix_section = ConfusionMatrixSection {
		false_negatives,
		false_positives,
		true_negatives,
		true_positives,
		class: class.clone(),
	};
	let page = Page {
		id: model_id.to_string(),
		model_layout_info,
		class,
		classes,
		confusion_matrix_section,
		precision_recall_section,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
