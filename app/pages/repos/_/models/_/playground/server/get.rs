use crate::page::{
	EnumFieldProps, FieldProps, FormProps, Inner, NumberFieldProps, Page, PageProps,
	TextFieldProps, UnknownFieldProps,
};
use html::html;
use num::ToPrimitive;
use std::collections::BTreeMap;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model_bytes,
	predict::{
		compute_feature_contributions_chart_series, compute_input_table_props,
		BinaryClassificationPredictOutputProps, MulticlassClassificationPredictOutputProps,
		PredictOutputInnerProps, PredictOutputProps, RegressionPredictOutputProps,
	},
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::{get_model_layout_props, ModelNavItem};
use tangram_core::predict::{PredictInputValue, PredictOptions};
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
	let model_layout_props =
		get_model_layout_props(&mut db, context, model_id, ModelNavItem::Playground).await?;
	let inner = compute_inner(model, search_params);
	let props = PageProps {
		model_layout_props,
		inner,
	};
	let html = html!(<Page {props} />).render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}

fn compute_inner(
	model: tangram_model::ModelReader,
	search_params: Option<BTreeMap<String, String>>,
) -> Inner {
	let input = predict_input_from_search_params(search_params);
	if let Some(input) = input {
		Inner::Output(compute_predict_output_props(model, input))
	} else {
		Inner::Form(compute_form_props(model, &input))
	}
}

fn compute_predict_output_props(
	model: tangram_model::ModelReader,
	input: tangram_core::predict::PredictInput,
) -> PredictOutputProps {
	let input_table = compute_input_table_props(model, &input);
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
			let feature_contributions_chart_series = compute_feature_contributions_chart_series(
				"output".to_owned(),
				feature_contributions,
			);
			PredictOutputInnerProps::Regression(RegressionPredictOutputProps {
				feature_contributions_chart_series,
				value: output.value,
			})
		}
		tangram_core::predict::PredictOutput::BinaryClassification(output) => {
			let feature_contributions = output.feature_contributions.unwrap();
			let feature_contributions_chart_series = compute_feature_contributions_chart_series(
				"output".to_owned(),
				feature_contributions,
			);
			PredictOutputInnerProps::BinaryClassification(BinaryClassificationPredictOutputProps {
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
			PredictOutputInnerProps::MulticlassClassification(
				MulticlassClassificationPredictOutputProps {
					class_name: output.class_name,
					feature_contributions_chart_series,
					probabilities: output.probabilities.into_iter().collect(),
					probability: output.probability,
				},
			)
		}
	};
	PredictOutputProps { inner, input_table }
}

// Convert the search params to predict input.
fn predict_input_from_search_params(
	search_params: Option<BTreeMap<String, String>>,
) -> Option<tangram_core::predict::PredictInput> {
	search_params
		.map(|search_params| {
			search_params
				.into_iter()
				.map(|(key, value)| (key, PredictInputValue::String(value)))
				.collect()
		})
		.map(tangram_core::predict::PredictInput)
}

fn compute_form_props(
	model: tangram_model::ModelReader,
	input: &Option<tangram_core::predict::PredictInput>,
) -> FormProps {
	let column_stats = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			regressor.overall_column_stats()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			binary_classifier.overall_column_stats()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			multiclass_classifier.overall_column_stats()
		}
	};
	let fields = column_stats
		.iter()
		.map(|column_stats| match column_stats {
			tangram_model::ColumnStatsReader::UnknownColumn(column_stats) => {
				let column_stats = column_stats.read();
				let name = column_stats.column_name().to_owned();
				let value = input
					.as_ref()
					.and_then(|s| s.0.get(&name))
					.cloned()
					.unwrap_or_else(|| PredictInputValue::String("".to_owned()))
					.into();
				FieldProps::Unknown(UnknownFieldProps { name, value })
			}
			tangram_model::ColumnStatsReader::NumberColumn(column_stats) => {
				let column_stats = column_stats.read();
				let name = column_stats.column_name().to_owned();
				let value = input
					.as_ref()
					.and_then(|s| s.0.get(&name))
					.cloned()
					.unwrap_or_else(|| {
						PredictInputValue::Number(column_stats.mean().to_f64().unwrap())
					})
					.into();
				FieldProps::Number(NumberFieldProps {
					name,
					max: column_stats.max(),
					min: column_stats.min(),
					p25: column_stats.p25(),
					p50: column_stats.p50(),
					p75: column_stats.p75(),
					value,
				})
			}
			tangram_model::ColumnStatsReader::EnumColumn(column_stats) => {
				let column_stats = column_stats.read();
				let histogram = &column_stats.histogram();
				let options = histogram.iter().map(|(key, _)| key.to_owned()).collect();
				let histogram = histogram
					.iter()
					.map(|(key, value)| (key.to_owned(), value))
					.collect::<Vec<_>>();
				let name = column_stats.column_name().to_owned();
				let compute_mode = || {
					histogram
						.iter()
						.max_by(|a, b| a.1.cmp(&b.1))
						.unwrap()
						.0
						.to_owned()
				};
				let value = input
					.as_ref()
					.and_then(|s| s.0.get(&name))
					.cloned()
					.unwrap_or_else(|| PredictInputValue::String(compute_mode()))
					.into();
				FieldProps::Enum(EnumFieldProps {
					name,
					options,
					value,
					histogram,
				})
			}
			tangram_model::ColumnStatsReader::TextColumn(column_stats) => {
				let column_stats = column_stats.read();
				let name = column_stats.column_name().to_owned();
				let value = input
					.as_ref()
					.and_then(|s| s.0.get(&name))
					.cloned()
					.unwrap_or_else(|| PredictInputValue::String("".to_owned()))
					.into();
				FieldProps::Text(TextFieldProps { name, value })
			}
		})
		.collect();
	FormProps { fields }
}
