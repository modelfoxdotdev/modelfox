use crate::page::{
	BinaryClassifier, ClassMetrics, ConfusionMatrixSection, Inner, MulticlassClassifier, Page,
	Regressor,
};
use anyhow::{bail, Result};
use pinwheel::prelude::*;
use std::sync::Arc;
use modelfox_app_context::Context;
use modelfox_app_core::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	user::{authorize_user, authorize_user_for_model},
};
use modelfox_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use modelfox_id::Id;

pub async fn get(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let model_id = if let ["repos", _, "models", model_id, "training_metrics", ""] =
		path_components(request).as_slice()
	{
		model_id.to_owned()
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
	let model = modelfox_model::from_bytes(&bytes)?;
	let inner = match model.inner() {
		modelfox_model::ModelInnerReader::Regressor(regressor) => {
			Inner::Regressor(build_inner_regressor(regressor.read()))
		}
		modelfox_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			Inner::BinaryClassifier(build_inner_binary_classifier(binary_classifier.read()))
		}
		modelfox_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			Inner::MulticlassClassifier(build_inner_multiclass_classifier(
				multiclass_classifier.read(),
			))
		}
	};
	let model_layout_info =
		model_layout_info(&mut db, app, model_id, ModelNavItem::TrainingMetrics).await?;
	let page = Page {
		id: model_id.to_string(),
		inner,
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

fn build_inner_regressor(model: modelfox_model::RegressorReader) -> Regressor {
	let warning = if model.baseline_metrics().rmse() > model.test_metrics().rmse() {
		Some("Baseline RMSE is lower! Your model performs worse than if it were just guessing the mean of the target column.".into())
	} else {
		None
	};
	Regressor {
		warning,
		rmse: model.test_metrics().rmse(),
		baseline_rmse: model.baseline_metrics().rmse(),
		mse: model.test_metrics().mse(),
		baseline_mse: model.baseline_metrics().mse(),
	}
}

fn build_inner_binary_classifier(model: modelfox_model::BinaryClassifierReader) -> BinaryClassifier {
	let test_metrics = model.test_metrics();
	let default_threshold_test_metrics = test_metrics.default_threshold();
	let default_threshold_baseline_metrics = model.baseline_metrics().default_threshold();
	let warning = if default_threshold_baseline_metrics.accuracy()
		> default_threshold_test_metrics.accuracy()
	{
		Some("Baseline Accuracy is higher! Your model performs worse than if it always predicted the majority class.".into())
	} else {
		None
	};
	let true_negatives = default_threshold_test_metrics.true_negatives();
	let true_positives = default_threshold_test_metrics.true_positives();
	let false_negatives = default_threshold_test_metrics.false_negatives();
	let false_positives = default_threshold_test_metrics.false_positives();
	let confusion_matrix_section = ConfusionMatrixSection {
		false_negatives,
		false_positives,
		true_negatives,
		true_positives,
		class: model.positive_class().to_owned(),
	};
	BinaryClassifier {
		warning,
		accuracy: default_threshold_test_metrics.accuracy(),
		baseline_accuracy: default_threshold_baseline_metrics.accuracy(),
		auc_roc: model.test_metrics().auc_roc(),
		precision: default_threshold_test_metrics.precision().unwrap(),
		recall: default_threshold_test_metrics.recall().unwrap(),
		f1_score: default_threshold_test_metrics.f1_score().unwrap(),
		positive_class: model.positive_class().to_owned(),
		negative_class: model.negative_class().to_owned(),
		target_column_name: model.target_column_name().to_owned(),
		confusion_matrix_section,
	}
}

fn build_inner_multiclass_classifier(
	model: modelfox_model::MulticlassClassifierReader,
) -> MulticlassClassifier {
	let classes = model
		.classes()
		.iter()
		.map(|class| class.to_owned())
		.collect::<Vec<_>>();
	let class_metrics = model
		.test_metrics()
		.class_metrics()
		.iter()
		.map(|class_metrics| ClassMetrics {
			precision: class_metrics.precision(),
			recall: class_metrics.recall(),
		})
		.collect::<Vec<ClassMetrics>>();
	let baseline_metrics = model.baseline_metrics();
	let test_metrics = model.test_metrics();
	let warning = if baseline_metrics.accuracy() > test_metrics.accuracy() {
		Some("Baseline Accuracy is higher! Your model performs worse than if it always predicted the majority class.".into())
	} else {
		None
	};
	MulticlassClassifier {
		warning,
		accuracy: model.test_metrics().accuracy(),
		baseline_accuracy: model.baseline_metrics().accuracy(),
		class_metrics,
		classes,
	}
}
