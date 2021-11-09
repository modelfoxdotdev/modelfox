use crate::ServeArgs;
use anyhow::Result;
use axum::{
	async_trait,
	extract::Extension,
	http::StatusCode,
	response::IntoResponse,
	routing::{get, post},
	AddExtensionLayer, Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tangram_core::predict::Model;

#[tokio::main]
pub async fn serve(args: ServeArgs) -> Result<()> {
	let bytes = std::fs::read(&args.model)?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model = Model::from(model);
	let app = app(model);
	let addr = format!("{}:{}", args.host, args.port).parse()?;

	tracing::info!(
		"Servingmodel at {} at address {}:{}",
		args.model.display(),
		args.host,
		args.port
	);

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await?;
	Ok(())
}

fn app(model: Model) -> Router {
	let model_provider = Arc::new(BaseModelProvider(model)) as DynModelProvider;

	Router::new()
		.route("/", get(root))
		.layer(AddExtensionLayer::new(model_provider))
}

async fn root(Extension(model_provider): Extension<DynModelProvider>) -> Json<Value> {
	let id = model_provider.id().await;
	Json(json!({ "model_id": id }))
}

struct BaseModelProvider(Model);

#[async_trait]
impl ModelProvider for BaseModelProvider {
	async fn id(&self) -> String {
		self.0.id.clone()
	}
}

#[async_trait]
trait ModelProvider {
	async fn id(&self) -> String;
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
	async fn test_model_id() {
		let bytes = std::fs::read("heart_disease.tangram").unwrap();
		let model = tangram_model::from_bytes(&bytes).unwrap();
		let model = Model::from(model);

		let app = app(model);

		let response = app
			.oneshot(
				Request::builder()
					.method(http::Method::GET)
					.uri("/")
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
}
