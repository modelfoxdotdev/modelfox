use crate::page::{Found, Inner, NotFound, Page};
use chrono::prelude::*;
use chrono_tz::Tz;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	path_components,
	predict::{
		compute_feature_contributions_chart_series, compute_input_table,
		BinaryClassificationPredictOutput, MulticlassClassificationPredictOutput, PredictOutput,
		PredictOutputInner, RegressionPredictOutput,
	},
	timezone::get_timezone,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{model_layout_info, ModelNavItem};
use tangram_core::predict::{PredictInput, PredictOptions};
use tangram_error::{err, Result};
use tangram_id::Id;

pub async fn get(
	context: Arc<Context>,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let (model_id, identifier) = if let ["repos", _, "models", model_id, "production_predictions", "predictions", identifier] =
		path_components(&request).as_slice()
	{
		(model_id.to_owned(), identifier.to_owned())
	} else {
		return Err(err!("unexpected path"));
	};
	let timezone = get_timezone(&request);
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
	let bytes = get_model_bytes(&context.storage, model_id).await?;
	let model = tangram_model::from_bytes(&bytes)?;
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model_layout_info = model_layout_info(
		&mut db,
		&context,
		model_id,
		ModelNavItem::ProductionPredictions,
	)
	.await?;
	let row = sqlx::query(
		"
			select
				date,
				identifier,
				input,
				output
			from predictions
				where
					model_id = $1
				and identifier = $2
			order by date
			limit 10
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_optional(&mut *db)
	.await?;
	let inner = match row {
		Some(row) => {
			let date: i64 = row.get(0);
			let date: DateTime<Tz> = Utc.timestamp(date, 0).with_timezone(&timezone);
			let input: String = row.get(2);
			let input: PredictInput = serde_json::from_str(&input)?;
			let input_table = compute_input_table(model, &input);
			let bytes = get_model_bytes(&context.storage, model_id).await?;
			let model = tangram_model::from_bytes(&bytes)?;
			let predict_model = tangram_core::predict::Model::from(model);
			let options = PredictOptions {
				compute_feature_contributions: true,
				..Default::default()
			};
			let mut output = tangram_core::predict::predict(&predict_model, &[input], &options);
			let output = output.remove(0);
			let inner = match output {
				tangram_core::predict::PredictOutput::Regression(output) => {
					let feature_contributions = output.feature_contributions.unwrap();
					let feature_contributions_chart_series =
						compute_feature_contributions_chart_series(
							"output".to_owned(),
							feature_contributions,
						);
					PredictOutputInner::Regression(RegressionPredictOutput {
						feature_contributions_chart_series,
						value: output.value,
					})
				}
				tangram_core::predict::PredictOutput::BinaryClassification(output) => {
					let feature_contributions = output.feature_contributions.unwrap();
					let feature_contributions_chart_series =
						compute_feature_contributions_chart_series(
							"output".to_owned(),
							feature_contributions,
						);
					PredictOutputInner::BinaryClassification(BinaryClassificationPredictOutput {
						class_name: output.class_name,
						feature_contributions_chart_series,
						probability: output.probability,
					})
				}
				tangram_core::predict::PredictOutput::MulticlassClassification(output) => {
					let feature_contributions = output.feature_contributions.unwrap();
					let feature_contributions_chart_series = feature_contributions
						.into_iter()
						.map(|(class, feature_contributions)| {
							compute_feature_contributions_chart_series(class, feature_contributions)
						})
						.collect();
					PredictOutputInner::MulticlassClassification(
						MulticlassClassificationPredictOutput {
							class_name: output.class_name,
							feature_contributions_chart_series,
							probabilities: output.probabilities.into_iter().collect(),
							probability: output.probability,
						},
					)
				}
			};
			Inner::Found(Box::new(Found {
				date: date.to_string(),
				identifier: identifier.to_owned(),
				predict_output: PredictOutput { inner, input_table },
			}))
		}
		None => Inner::NotFound(Box::new(NotFound {
			identifier: identifier.to_owned(),
		})),
	};

	let page = Page {
		model_layout_info,
		identifier: identifier.to_owned(),
		inner,
	};
	let html = html(page);
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
