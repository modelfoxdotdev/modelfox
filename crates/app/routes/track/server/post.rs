use anyhow::Result;
use std::{collections::BTreeMap, sync::Arc};
use tangram_app_context::Context;
use tangram_app_core::{
	app::{handle_prediction_monitor_event, handle_true_value_monitor_event},
	error::{bad_request, service_unavailable},
	monitor_event::MonitorEvent,
};
use tracing::error;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum MonitorEventSet {
	Single(MonitorEvent),
	Multiple(Vec<MonitorEvent>),
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = Arc::clone(request.extensions().get::<Arc<Context>>().unwrap());
	let app = &context.app;
	let bytes = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(bytes) => bytes,
		Err(e) => {
			error!(%e);
			return Ok(bad_request());
		}
	};
	let monitor_events: MonitorEventSet = match serde_json::from_slice(&bytes) {
		Ok(monitor_events) => monitor_events,
		Err(e) => {
			error!(%e);
			return Ok(bad_request());
		}
	};
	let monitor_events = match monitor_events {
		MonitorEventSet::Single(monitor_event) => vec![monitor_event],
		MonitorEventSet::Multiple(monitor_event) => monitor_event,
	};
	let mut db = match app.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let mut model_cache = BTreeMap::new();
	for monitor_event in monitor_events {
		match monitor_event {
			MonitorEvent::Prediction(monitor_event) => {
				let handle_prediction_result =
					handle_prediction_monitor_event(&mut db, &app.storage, &mut model_cache, monitor_event).await;
				if let Err(e) = handle_prediction_result {
					error!(%e);
					return Ok(bad_request());
				}
			}
			MonitorEvent::TrueValue(monitor_event) => {
				let handle_true_value_result = handle_true_value_monitor_event(
					&mut db,
					&app.storage,
					&mut model_cache,
					monitor_event,
				)
				.await;
				if handle_true_value_result.is_err() {
					return Ok(bad_request());
				}
			}
		}
	}
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::ACCEPTED)
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
