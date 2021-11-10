use crate::ServeArgs;
use anyhow::Result;
use axum::{
	async_trait,
	extract::Extension,
	handler::Handler,
	http::{HeaderMap, Request, Response, StatusCode},
	response::IntoResponse,
	routing::{get, post},
	AddExtensionLayer, Json, Router,
};
use bytes::Bytes;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tangram_core::predict::{Model, PredictInput};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

#[tokio::main]
pub async fn serve(args: ServeArgs) -> Result<()> {
	let bytes = std::fs::read(&args.model)?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model = Model::from(model);
	let app = app(model);
	let addr = format!("{}:{}", args.host, args.port).parse()?;

	tracing::info!("Serving model from {} at {}", args.model.display(), addr);

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await?;
	Ok(())
}

fn app(model: Model) -> Router {
	let model_provider = Arc::new(BaseModelProvider(model)) as DynModelProvider;
	Router::new()
		.route("/", get(root))
		.route("/id", get(id))
		.route("/predict", post(predict))
		.layer(AddExtensionLayer::new(model_provider))
		.layer(
			TraceLayer::new_for_http()
				.make_span_with(|request: &Request<_>| tracing::debug_span!("http-request"))
				.on_request(|request: &Request<_>, _span: &Span| {
					tracing::debug!("started {} {}", request.method(), request.uri().path())
				})
				.on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
					tracing::debug!("response generated in {:?}", latency)
				})
				.on_body_chunk(|chunk: &Bytes, latency: Duration, _span: &Span| {
					tracing::debug!("sending {} bytes", chunk.len())
				})
				.on_eos(
					|trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
						tracing::debug!("stream closed after {:?}", stream_duration)
					},
				)
				.on_failure(
					|error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
						tracing::warn!("something went wrong")
					},
				),
		)
		.fallback(handler_404.into_service())
}

async fn root() -> &'static str {
	"Model loaded!"
}

async fn id(Extension(model_provider): Extension<DynModelProvider>) -> Json<Value> {
	let id = model_provider.id().await;
	Json(json!({ "model_id": id }))
}

async fn predict(
	Json(payload): Json<PredictInput>,
	Extension(model_provider): Extension<DynModelProvider>,
) -> Json<Value> {
	// Convert payload to PredictInput
	// Run prediction
	// Return the Vec<PredictOutput> - probably serde_json::to_vec()
	unimplemented!()
}

async fn handler_404() -> impl IntoResponse {
	(StatusCode::NOT_FOUND, "Not found")
}

struct BaseModelProvider(Model);

#[async_trait]
impl ModelProvider for BaseModelProvider {
	async fn id(&self) -> String {
		self.0.id.clone()
	}
	async fn predict(&self, predict_input: PredictInput) {
		unimplemented!()
	}
}

#[async_trait]
trait ModelProvider {
	async fn id(&self) -> String;
	async fn predict(&self, _: PredictInput);
}

type DynModelProvider = Arc<dyn ModelProvider + Send + Sync>;

//#[derive(Debug)]
//enum ServeError {
//    // do I need this?
//}

#[cfg(test)]
mod test {
	use super::*;
	use axum::{
		body::Body,
		http::{self, Request, StatusCode},
	};
	use pretty_assertions::assert_eq;
	use tower::ServiceExt;

	#[tokio::test]
	async fn test_four_oh_four() {
		let bytes = std::fs::read("heart_disease.tangram").unwrap();
		let model = tangram_model::from_bytes(&bytes).unwrap();
		let model = Model::from(model);

		let app = app(model);

		let response = app
			.oneshot(
				Request::builder()
					.method(http::Method::GET)
					.uri("/nonsense")
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::NOT_FOUND);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		assert_eq!(body, "Not found");
	}

	#[tokio::test]
	async fn test_model_id() {
		let bytes = std::fs::read("heart_disease.tangram").unwrap();
		let model = tangram_model::from_bytes(&bytes).unwrap();
		let model = Model::from(model);

		let app = app(model);

		let response = app
			.oneshot(
				Request::builder()
					.method(http::Method::GET)
					.uri("/id")
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::OK);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		let body: Value = serde_json::from_slice(&body).unwrap();
		assert_eq!(
			body,
			json!({ "model_id": "4df212cc47134706f5f6e3c78e889c0d" })
		);
	}

	#[tokio::test]
	async fn test_predict() {
		let bytes = std::fs::read("heart_disease.tangram").unwrap();
		let model = tangram_model::from_bytes(&bytes).unwrap();
		let model = Model::from(model);

		let app = app(model);

		let response = app
			.oneshot(
				Request::builder()
					.method(http::Method::POST)
					.uri("/predict")
					.header(http::header::CONTENT_TYPE, "application/json")
					.body(Body::from(
						serde_json::to_vec(&json!({
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
						}))
						.unwrap(),
					))
					.unwrap(),
			)
			.await
			.unwrap();

		assert_eq!(response.status(), StatusCode::OK);

		let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		let body: Value = serde_json::from_slice(&body).unwrap();
		assert_eq!(
			body,
			json!({ "class_name": "Positive", "probability": 0.5604339838027954 })
		);
	}
}
