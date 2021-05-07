use crate::page::{Page, PageProps, RocCurveData};
use html::html;
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
	let props = match model.inner() {
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			let test_metrics = binary_classifier.test_metrics();
			let roc_curve_data = test_metrics
				.thresholds()
				.iter()
				.map(|class_metrics| RocCurveData {
					false_positive_rate: class_metrics.false_positive_rate(),
					true_positive_rate: class_metrics.true_positive_rate(),
				})
				.collect();
			let auc_roc = test_metrics.auc_roc();
			let model_layout_props =
				get_model_layout_props(&mut db, context, model_id, ModelNavItem::TrainingMetrics)
					.await?;
			PageProps {
				id: model_id.to_string(),
				class: binary_classifier.positive_class().to_owned(),
				roc_curve_data,
				auc_roc,
				model_layout_props,
			}
		}
		_ => {
			return Ok(bad_request());
		}
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
