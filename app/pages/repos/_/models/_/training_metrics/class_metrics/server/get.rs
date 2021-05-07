use crate::page::{ConfusionMatrixSectionProps, Page, PageProps, PrecisionRecallSectionProps};
use html::html;
use std::collections::BTreeMap;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{get_model_layout_props, ModelNavItem};
use tangram_error::Result;
use tangram_id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
	search_params: Option<BTreeMap<String, String>>,
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
	let multiclass_classifier = match model.inner() {
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read()
		}
		_ => return Ok(bad_request()),
	};
	let class = search_params.map(|s| s.get("class").unwrap().clone());
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
	let model_layout_props =
		get_model_layout_props(&mut db, context, model_id, ModelNavItem::TrainingMetrics).await?;
	let precision_recall_section_props = PrecisionRecallSectionProps {
		f1_score,
		precision,
		recall,
		class: class.clone(),
	};
	let confusion_matrix_section_props = ConfusionMatrixSectionProps {
		false_negatives,
		false_positives,
		true_negatives,
		true_positives,
		class: class.clone(),
	};
	let props = PageProps {
		id: model_id.to_string(),
		model_layout_props,
		class,
		classes,
		confusion_matrix_section_props,
		precision_recall_section_props,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
