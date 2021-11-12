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

use crate::{backtrace_server, ServeArgs};
use bytes::Buf;
use hyper::http;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tangram_core::predict::{PredictInput, PredictOptions, PredictOutput};

#[tokio::main]
pub async fn serve(args: ServeArgs) -> anyhow::Result<()> {
	// Read model and create context
	let bytes = std::fs::read(&args.model)?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model = tangram_core::predict::Model::from(model);
	let context = Arc::new(model);

	// Parse address
	let addr = std::net::SocketAddr::new(args.address.parse()?, args.port);

	tracing::info!("Serving model from {} at {}", args.model.display(), addr);
	backtrace_server::serve(addr, context, handle).await?;
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
		&inputs.0,
		&PredictOptions::default(),
	));
	let json = serde_json::to_string(&outputs).unwrap();
	tracing::debug!("sending {} bytes", json.len());
	http::Response::builder()
		.body(hyper::Body::from(json))
		.unwrap()
}

#[derive(Deserialize)]
struct PredictInputs(Vec<PredictInput>);

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
	async fn test_predict() {
		let payload = json!([{
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
		}]);

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
			hyper::body::Bytes::from(
				"bad request: invalid type: map, expected a sequence at line 1 column 1"
			)
		);
	}
}
