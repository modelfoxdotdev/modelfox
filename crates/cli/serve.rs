//! This module exposes an HTTP endpoint for predicting against a pregenerated model
//!
//! Start the server, pointing to a valid `.tangram` file:
//! ```not-rust
//! $ tangram serve --model heart_disease.tangram
//! ```
//!
//! Make a request:
//! ```not-rust
//! $ curl -X POST 127.0.0.1:8080/predict -H 'Content-Type: application/json' -d '[{"age": 63.0,"gender": "male","chest_pain": "typical angina","resting_blood_pressure": 145.0,"cholesterol": 233.0,"fasting_blood_sugar_greater_than_120": "true","resting_ecg_result": "probable or definite left ventricular hypertrophy","exercise_max_heart_rate": 150.0,"exercise_induced_angina": "no","exercise_st_depression": 2.3,"exercise_st_slope": "downsloping","fluoroscopy_vessels_colored": "0","thallium_stress_test": "fixed defect"}]'
//! [{"type":"binary_classification","class_name":"Positive","probability":0.560434,"feature_contributions":null}]
//! ```

use crate::ServeArgs;
use anyhow::Result;
use bytes::Buf;
use hyper::http;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tangram_core::predict::{PredictInput, PredictOptions, PredictOutput};

#[tokio::main]
pub async fn serve(args: ServeArgs) -> Result<()> {
	// Read model and create context
	let bytes = std::fs::read(&args.model)?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model = tangram_core::predict::Model::from(model);
	let context = Arc::new(model);

	// Parse address
	let addr = std::net::SocketAddr::new(args.address.parse()?, args.port);

	tracing::info!("Serving model from {}", args.model.display());
	tangram_serve::serve(addr, context, handle).await?;
	Ok(())
}

fn bad_request(msg: &str) -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::BAD_REQUEST)
		.body(hyper::Body::from(format!("bad request: {}", msg)))
		.unwrap()
}

fn not_found() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::NOT_FOUND)
		.body(hyper::Body::from("not found"))
		.unwrap()
}

async fn predict(request: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
	let context: Arc<tangram_core::predict::Model> =
		Arc::clone(request.extensions().get().unwrap());
	let body = request.into_body();
	let body_bytes = hyper::body::aggregate(body).await.unwrap();
	let inputs: PredictInputs = match serde_json::from_reader(body_bytes.reader()) {
		Ok(inputs) => inputs,
		Err(e) => {
			let msg = e.to_string();
			tracing::debug!("sending {} bytes", msg.len());
			return bad_request(&msg);
		}
	};
	let outputs = PredictOutputs(tangram_core::predict::predict(
		&*context,
		&inputs.inputs,
		&inputs.options.unwrap_or_default(),
	));
	let json = serde_json::to_string(&outputs).unwrap();
	tracing::debug!("sending {} bytes", json.len());
	http::Response::builder()
		.body(hyper::Body::from(json))
		.unwrap()
}

#[derive(Deserialize)]
struct PredictInputs {
	inputs: Vec<PredictInput>,
	options: Option<PredictOptions>,
}

#[derive(Serialize)]
struct PredictOutputs(Vec<PredictOutput>);

async fn handle(request: http::Request<hyper::Body>) -> http::Response<hyper::Body> {
	match (request.method(), request.uri().path()) {
		(&hyper::Method::POST, "/predict") => predict(request).await,
		_ => not_found(),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use pretty_assertions::assert_eq;
	use serde_json::{json, Value};

	fn test_model() -> tangram_core::predict::Model {
		let bytes = std::fs::read("heart_disease.tangram").unwrap();
		let model = tangram_model::from_bytes(&bytes).unwrap();
		tangram_core::predict::Model::from(model)
	}

	#[tokio::test]
	async fn test_four_oh_four() {
		let mut request = hyper::Request::builder()
			.method(http::Method::GET)
			.uri("/nonsense")
			.body(hyper::Body::empty())
			.unwrap();
		let context = Arc::new(test_model());
		request.extensions_mut().insert(Arc::clone(&context));
		let response = handle(request).await;

		assert_eq!(response.status(), http::status::StatusCode::NOT_FOUND);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		assert_eq!(body, "not found");
	}

	#[tokio::test]
	async fn test_predict_default_options() {
		let payload = json!({ "inputs": [{
					"age": 63.0,
					"gender": "male",
					"chest_pain": "typical angina",
					"resting_blood_pressure": 145.0,
					"cholesterol": 233.0,
					"fasting_blood_sugar_greater_than_120": "true",
					"resting_ecg_result": "probable or definite left ventricular hypertrophy",
					"exercise_max_heart_rate": 150.0,
					"exercise_induced_angina": "no",
					"exercise_st_depression": 2.3,
					"exercise_st_slope": "downsloping",
					"fluoroscopy_vessels_colored": "0",
					"thallium_stress_test": "fixed defect"
		}]});

		let mut request = hyper::Request::builder()
			.method(http::Method::POST)
			.uri("/predict")
			.header(http::header::CONTENT_TYPE, "application/json")
			.body(hyper::Body::from(payload.to_string()))
			.unwrap();

		let context = Arc::new(test_model());
		request.extensions_mut().insert(Arc::clone(&context));
		let response = handle(request).await;

		assert_eq!(response.status(), http::status::StatusCode::OK);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		let body: Value = serde_json::from_slice(&body).unwrap();
		let expected = json!(
		[
			{
				"class_name": "Positive",
				"feature_contributions": null,
				"probability": 0.560434,
				"type": "binary_classification"
			}
		]);
		assert_eq!(body, expected);
	}

	#[tokio::test]
	async fn test_predict_with_options() {
		let payload = json!({ "inputs": [{
					"age": 63.0,
					"gender": "male",
					"chest_pain": "typical angina",
					"resting_blood_pressure": 145.0,
					"cholesterol": 233.0,
					"fasting_blood_sugar_greater_than_120": "true",
					"resting_ecg_result": "probable or definite left ventricular hypertrophy",
					"exercise_max_heart_rate": 150.0,
					"exercise_induced_angina": "no",
					"exercise_st_depression": 2.3,
					"exercise_st_slope": "downsloping",
					"fluoroscopy_vessels_colored": "0",
					"thallium_stress_test": "fixed defect"
		}],
		"options": {
			"threshold": 0.4,
			"compute_feature_contributions": true
		}});

		let mut request = hyper::Request::builder()
			.method(http::Method::POST)
			.uri("/predict")
			.header(http::header::CONTENT_TYPE, "application/json")
			.body(hyper::Body::from(payload.to_string()))
			.unwrap();

		let context = Arc::new(test_model());
		request.extensions_mut().insert(Arc::clone(&context));
		let response = handle(request).await;

		assert_eq!(response.status(), http::status::StatusCode::OK);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		let body: Value = serde_json::from_slice(&body).unwrap();
		let expected = json!([{"type":"binary_classification","class_name":"Positive","probability":0.560434,"feature_contributions":{"baseline_value":0.20128278,"output_value":0.24292351,"entries":[{"type":"normalized","column_name":"age","feature_value":0.9329086,"feature_contribution_value":0.12275841},{"type":"one_hot_encoded","column_name":"gender","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"gender","variant":"female","feature_value":false,"feature_contribution_value":0.12700649},{"type":"one_hot_encoded","column_name":"gender","variant":"male","feature_value":true,"feature_contribution_value":0.119262576},{"type":"one_hot_encoded","column_name":"chest_pain","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"chest_pain","variant":"asymptomatic","feature_value":false,"feature_contribution_value":-0.3736595},{"type":"one_hot_encoded","column_name":"chest_pain","variant":"atypical angina","feature_value":false,"feature_contribution_value":0.0072757024},{"type":"one_hot_encoded","column_name":"chest_pain","variant":"non-angina pain","feature_value":false,"feature_contribution_value":0.10761015},{"type":"one_hot_encoded","column_name":"chest_pain","variant":"typical angina","feature_value":true,"feature_contribution_value":-0.19594865},{"type":"normalized","column_name":"resting_blood_pressure","feature_value":0.82200927,"feature_contribution_value":0.115394905},{"type":"normalized","column_name":"cholesterol","feature_value":-0.23350535,"feature_contribution_value":-0.035092965},{"type":"one_hot_encoded","column_name":"fasting_blood_sugar_greater_than_120","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"fasting_blood_sugar_greater_than_120","variant":"false","feature_value":false,"feature_contribution_value":-0.052730173},{"type":"one_hot_encoded","column_name":"fasting_blood_sugar_greater_than_120","variant":"true","feature_value":true,"feature_contribution_value":-0.074512005},{"type":"one_hot_encoded","column_name":"resting_ecg_result","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"resting_ecg_result","variant":"ST-T wave abnormality","feature_value":false,"feature_contribution_value":-0.00006990708},{"type":"one_hot_encoded","column_name":"resting_ecg_result","variant":"normal","feature_value":false,"feature_contribution_value":0.07310219},{"type":"one_hot_encoded","column_name":"resting_ecg_result","variant":"probable or definite left ventricular hypertrophy","feature_value":true,"feature_contribution_value":0.05366865},{"type":"normalized","column_name":"exercise_max_heart_rate","feature_value":0.03279825,"feature_contribution_value":-0.01721257},{"type":"one_hot_encoded","column_name":"exercise_induced_angina","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"exercise_induced_angina","variant":"no","feature_value":true,"feature_contribution_value":-0.079578854},{"type":"one_hot_encoded","column_name":"exercise_induced_angina","variant":"yes","feature_value":false,"feature_contribution_value":-0.070175245},{"type":"normalized","column_name":"exercise_st_depression","feature_value":1.1320461,"feature_contribution_value":0.54184896},{"type":"one_hot_encoded","column_name":"exercise_st_slope","variant":null,"feature_value":false,"feature_contribution_value":0.0},{"type":"one_hot_encoded","column_name":"exercise_st_slope","variant":"downsloping","feature_value":true,"feature_contribution_value":0.060956795},{"type":"one_hot_encoded","column_name":"exercise_st_slope","variant":"flat","feature_value":false,"feature_contribution_value":-0.10917909},{"type":"one_hot_encoded","column_name":"exercise_st_slope","variant":"upsloping","feature_value":false,"feature_contribution_value":0.1394263},{"type":"normalized","column_name":"fluoroscopy_vessels_colored","feature_value":-0.7464805,"feature_contribution_value":-0.56697065},{"type":"one_hot_encoded","column_name":"thallium_stress_test","variant":null,"feature_value":false,"feature_contribution_value":-0.00017653508},{"type":"one_hot_encoded","column_name":"thallium_stress_test","variant":"fixed defect","feature_value":true,"feature_contribution_value":0.0515052},{"type":"one_hot_encoded","column_name":"thallium_stress_test","variant":"normal","feature_value":false,"feature_contribution_value":0.34776008},{"type":"one_hot_encoded","column_name":"thallium_stress_test","variant":"reversible defect","feature_value":false,"feature_contribution_value":-0.25062957}]}}]
		);
		assert_eq!(body, expected);
	}

	#[tokio::test]
	async fn test_predict_bad_payload() {
		let bad_payload = json!({ "nonsense": "present" });

		let mut request = hyper::Request::builder()
			.method(http::Method::POST)
			.uri("/predict")
			.header(http::header::CONTENT_TYPE, "application/json")
			.body(hyper::Body::from(bad_payload.to_string()))
			.unwrap();
		let context = Arc::new(test_model());
		request.extensions_mut().insert(Arc::clone(&context));
		let response = handle(request).await;

		assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		assert_eq!(
			body,
			hyper::body::Bytes::from("bad request: missing field `inputs` at line 1 column 22")
		);
	}
}
