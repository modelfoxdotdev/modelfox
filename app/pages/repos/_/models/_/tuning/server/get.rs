use crate::page::Page;
use pinwheel::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_app_tuning_common::{Metrics, Tuning};
use tangram_error::{err, Result};
use tangram_id::Id;

pub async fn get(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let model_id =
		if let ["repos", _, "models", model_id, "tuning"] = *path_components(&request).as_slice() {
			model_id.to_owned()
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
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	let tuning = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(_) => None,
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let model = binary_classifier.read();
			let metrics: Vec<Metrics> = model
				.test_metrics()
				.thresholds()
				.iter()
				.map(|metrics| {
					let total = metrics.false_negatives() as f32
						+ metrics.false_positives() as f32
						+ metrics.true_positives() as f32
						+ metrics.true_negatives() as f32;
					Metrics {
						threshold: metrics.threshold(),
						precision: metrics.precision(),
						recall: metrics.recall(),
						accuracy: metrics.accuracy(),
						f1_score: metrics.f1_score(),
						false_negatives_fraction: metrics.false_negatives() as f32 / total,
						false_positives_fraction: metrics.false_positives() as f32 / total,
						true_negatives_fraction: metrics.true_negatives() as f32 / total,
						true_positives_fraction: metrics.true_positives() as f32 / total,
					}
				})
				.collect();
			let test_metrics = model.test_metrics();
			let default_threshold_metrics = test_metrics.default_threshold();
			let total = default_threshold_metrics.false_negatives() as f32
				+ default_threshold_metrics.false_positives() as f32
				+ default_threshold_metrics.true_negatives() as f32
				+ default_threshold_metrics.true_positives() as f32;
			let default_threshold_metrics = Metrics {
				threshold: default_threshold_metrics.threshold(),
				precision: default_threshold_metrics.precision(),
				recall: default_threshold_metrics.recall(),
				accuracy: default_threshold_metrics.accuracy(),
				f1_score: default_threshold_metrics.f1_score(),
				false_negatives_fraction: default_threshold_metrics.false_negatives() as f32
					/ total,
				false_positives_fraction: default_threshold_metrics.false_positives() as f32
					/ total,
				true_negatives_fraction: default_threshold_metrics.true_negatives() as f32 / total,
				true_positives_fraction: default_threshold_metrics.true_positives() as f32 / total,
			};
			Some(Tuning {
				default_threshold: 0.5,
				metrics,
				default_threshold_metrics,
				class: model.positive_class().to_owned(),
			})
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(_) => None,
	};
	let model_layout_info =
		model_layout_info(&mut db, &context, model_id, ModelNavItem::Tuning).await?;
	let page = Page {
		model_layout_info,
		tuning,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
