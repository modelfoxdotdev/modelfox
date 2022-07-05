//! This module runs an HTTP server for making predictions with a modelfox model.
//!
//! Start the server with a `.modelfox` file:
//! ```not-rust
//! $ modelfox serve --model heart_disease.modelfox
//! ```
//!
//! Make a request:
//! ```not-rust
//! $ curl -X POST http://localhost:8080/predict -H 'Content-Type: application/json' -d '{ "inputs": [{"age": 63.0,"gender": "male","chest_pain": "typical angina","resting_blood_pressure": 145.0,"cholesterol": 233.0,"fasting_blood_sugar_greater_than_120": "true","resting_ecg_result": "probable or definite left ventricular hypertrophy","exercise_max_heart_rate": 150.0,"exercise_induced_angina": "no","exercise_st_depression": 2.3,"exercise_st_slope": "downsloping","fluoroscopy_vessels_colored": "0","thallium_stress_test": "fixed defect"}]}'
//![{"type":"binary_classification","class_name":"Positive","probability":0.560434,"feature_contributions":null}]
//! ```

use crate::ServeArgs;
use anyhow::Result;
use bytes::Buf;
use hyper::http;
use modelfox_core::predict::{PredictInput, PredictOptions, PredictOutput};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[tokio::main]
pub async fn serve(args: ServeArgs) -> Result<()> {
	// Read model and create context
	let bytes = std::fs::read(&args.model)?;
	let model = modelfox_model::from_bytes(&bytes)?;
	let model = modelfox_core::predict::Model::from(model);
	let context = Arc::new(model);

	// Parse address
	let addr = std::net::SocketAddr::new(args.address.parse()?, args.port);

	tracing::info!("Serving model from {}", args.model.display());
	modelfox_serve::serve(addr, context, handle).await?;
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
	let context: Arc<modelfox_core::predict::Model> =
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
	let outputs = PredictOutputs(modelfox_core::predict::predict(
		&context,
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

	fn test_model() -> modelfox_core::predict::Model {
		let bytes = std::fs::read("../../heart_disease.modelfox").unwrap();
		let model = modelfox_model::from_bytes(&bytes).unwrap();
		modelfox_core::predict::Model::from(model)
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
		insta::assert_json_snapshot!(body, @r###"
  [
    {
      "class_name": "Positive",
      "feature_contributions": null,
      "probability": 0.56037986,
      "type": "binary_classification"
    }
  ]
  "###);
	}

	#[tokio::test]
	async fn test_predict_with_options() {
		let payload = json!({
			"inputs": [{
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
				"thallium_stress_test": "fixed defect",
			}],
			"options": {
				"threshold": 0.4,
				"compute_feature_contributions": true,
			},
		});

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
		insta::assert_json_snapshot!(body, @r###"
  [
    {
      "class_name": "Positive",
      "feature_contributions": {
        "baseline_value": 0.20130166,
        "entries": [
          {
            "column_name": "age",
            "feature_contribution_value": 0.12281105,
            "feature_value": 0.9329086,
            "type": "normalized"
          },
          {
            "column_name": "gender",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "gender",
            "feature_contribution_value": 0.12702154,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "female"
          },
          {
            "column_name": "gender",
            "feature_contribution_value": 0.11926158,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "male"
          },
          {
            "column_name": "chest_pain",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "chest_pain",
            "feature_contribution_value": -0.37382412,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "asymptomatic"
          },
          {
            "column_name": "chest_pain",
            "feature_contribution_value": 0.0072760214,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "atypical angina"
          },
          {
            "column_name": "chest_pain",
            "feature_contribution_value": 0.10765556,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "non-angina pain"
          },
          {
            "column_name": "chest_pain",
            "feature_contribution_value": -0.19594735,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "typical angina"
          },
          {
            "column_name": "resting_blood_pressure",
            "feature_contribution_value": 0.11539532,
            "feature_value": 0.82200927,
            "type": "normalized"
          },
          {
            "column_name": "cholesterol",
            "feature_contribution_value": -0.035099946,
            "feature_value": -0.23350535,
            "type": "normalized"
          },
          {
            "column_name": "fasting_blood_sugar_greater_than_120",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "fasting_blood_sugar_greater_than_120",
            "feature_contribution_value": -0.052793734,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "false"
          },
          {
            "column_name": "fasting_blood_sugar_greater_than_120",
            "feature_contribution_value": -0.07468745,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "true"
          },
          {
            "column_name": "resting_ecg_result",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "resting_ecg_result",
            "feature_contribution_value": -0.000069902,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "ST-T wave abnormality"
          },
          {
            "column_name": "resting_ecg_result",
            "feature_contribution_value": 0.073114455,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "normal"
          },
          {
            "column_name": "resting_ecg_result",
            "feature_contribution_value": 0.053654622,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "probable or definite left ventricular hypertrophy"
          },
          {
            "column_name": "exercise_max_heart_rate",
            "feature_contribution_value": -0.017210854,
            "feature_value": 0.03279825,
            "type": "normalized"
          },
          {
            "column_name": "exercise_induced_angina",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "exercise_induced_angina",
            "feature_contribution_value": -0.07958502,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "no"
          },
          {
            "column_name": "exercise_induced_angina",
            "feature_contribution_value": -0.0701625,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "yes"
          },
          {
            "column_name": "exercise_st_depression",
            "feature_contribution_value": 0.5418571,
            "feature_value": 1.1320461,
            "type": "normalized"
          },
          {
            "column_name": "exercise_st_slope",
            "feature_contribution_value": 0.0,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "exercise_st_slope",
            "feature_contribution_value": 0.06096074,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "downsloping"
          },
          {
            "column_name": "exercise_st_slope",
            "feature_contribution_value": -0.109167404,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "flat"
          },
          {
            "column_name": "exercise_st_slope",
            "feature_contribution_value": 0.1394413,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "upsloping"
          },
          {
            "column_name": "fluoroscopy_vessels_colored",
            "feature_contribution_value": -0.5669827,
            "feature_value": -0.7464805,
            "type": "normalized"
          },
          {
            "column_name": "thallium_stress_test",
            "feature_contribution_value": -0.00017653707,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": null
          },
          {
            "column_name": "thallium_stress_test",
            "feature_contribution_value": 0.05150451,
            "feature_value": true,
            "type": "one_hot_encoded",
            "variant": "fixed defect"
          },
          {
            "column_name": "thallium_stress_test",
            "feature_contribution_value": 0.34777343,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "normal"
          },
          {
            "column_name": "thallium_stress_test",
            "feature_contribution_value": -0.2506175,
            "feature_value": false,
            "type": "one_hot_encoded",
            "variant": "reversible defect"
          }
        ],
        "output_value": 0.24270391
      },
      "probability": 0.56037986,
      "type": "binary_classification"
    }
  ]
  "###);
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
