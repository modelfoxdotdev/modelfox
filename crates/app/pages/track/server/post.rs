use anyhow::{anyhow, Result};
use chrono::prelude::*;
use memmap::Mmap;
use num::ToPrimitive;
use sqlx::prelude::*;
use std::{collections::BTreeMap, sync::Arc};
use tangram_app_common::{
	error::{bad_request, service_unavailable},
	model::get_model_bytes,
	monitor_event::{
		BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
		NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
		TrueValueMonitorEvent,
	},
	storage::Storage,
	Context,
};
use tangram_app_production_metrics::ProductionMetrics;
use tangram_app_production_stats::ProductionStats;
use tangram_id::Id;
use tracing::error;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum MonitorEventSet {
	Single(MonitorEvent),
	Multiple(Vec<MonitorEvent>),
}

pub async fn post(request: &mut http::Request<hyper::Body>) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap().clone();
	let bytes = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(bytes) => bytes,
		Err(_) => return Ok(bad_request()),
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
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let mut model_cache = BTreeMap::new();
	for monitor_event in monitor_events {
		match monitor_event {
			MonitorEvent::Prediction(monitor_event) => {
				let handle_prediction_result = handle_prediction_monitor_event(
					&mut db,
					&context.storage,
					&mut model_cache,
					monitor_event,
				)
				.await;
				if handle_prediction_result.is_err() {
					return Ok(bad_request());
				}
			}
			MonitorEvent::TrueValue(monitor_event) => {
				let handle_true_value_result = handle_true_value_monitor_event(
					&mut db,
					&context.storage,
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

async fn handle_prediction_monitor_event(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	model_cache: &mut BTreeMap<Id, Mmap>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let bytes = match model_cache.get(&model_id) {
		Some(bytes) => bytes,
		None => {
			let model = get_model_bytes(data_storage, model_id).await?;
			model_cache.insert(model_id, model);
			model_cache.get(&model_id).unwrap()
		}
	};
	let model = tangram_model::from_bytes(bytes)?;
	write_prediction_monitor_event(&mut db, model_id, &monitor_event).await?;
	insert_or_update_production_stats_for_monitor_event(&mut db, model_id, model, monitor_event)
		.await?;
	Ok(())
}

async fn handle_true_value_monitor_event(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	data_storage: &Storage,
	model_cache: &mut BTreeMap<Id, Mmap>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let model_id = monitor_event.model_id;
	let bytes = match model_cache.get(&model_id) {
		Some(model) => model,
		None => {
			let model = get_model_bytes(data_storage, monitor_event.model_id).await?;
			model_cache.insert(model_id, model);
			model_cache.get(&model_id).unwrap()
		}
	};
	let model = tangram_model::from_bytes(bytes)?;
	write_true_value_monitor_event(&mut db, model_id, &monitor_event).await?;
	insert_or_update_production_metrics_for_monitor_event(&mut db, model_id, model, monitor_event)
		.await?;
	Ok(())
}

async fn write_prediction_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &PredictionMonitorEvent,
) -> Result<()> {
	let prediction_monitor_event_id = Id::generate();
	let identifier = monitor_event.identifier.as_string();
	let date = &monitor_event.date;
	let input = serde_json::to_string(&monitor_event.input)?;
	let output = serde_json::to_string(&monitor_event.output)?;
	let options = serde_json::to_string(&monitor_event.options)?;
	sqlx::query(
		"
			insert into predictions
				(id, model_id, date, identifier, input, options, output)
			values
				($1, $2, $3, $4, $5, $6, $7)
		",
	)
	.bind(&prediction_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
	.bind(&input)
	.bind(&options)
	.bind(&output)
	.execute(&mut *db)
	.await?;
	Ok(())
}

async fn write_true_value_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &TrueValueMonitorEvent,
) -> Result<()> {
	let true_value_monitor_event_id = Id::generate();
	let date = monitor_event.date;
	let identifier = monitor_event.identifier.as_string();
	let true_value = &monitor_event.true_value.to_string();
	sqlx::query(
		"
			insert into true_values
				(id, model_id, date, identifier, value)
			values
				($1, $2, $3, $4, $5)
		",
	)
	.bind(&true_value_monitor_event_id.to_string())
	.bind(&model_id.to_string())
	.bind(&date.timestamp())
	.bind(&identifier.to_string())
	.bind(&true_value.to_string())
	.execute(&mut *db)
	.await?;
	Ok(())
}

async fn insert_or_update_production_stats_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: tangram_model::ModelReader<'_>,
	monitor_event: PredictionMonitorEvent,
) -> Result<()> {
	let date = monitor_event.date;
	let hour = Utc
		.ymd(date.year(), date.month(), date.day())
		.and_hms(date.hour(), 0, 0);
	let rows = sqlx::query(
		"
			select
				data
			from production_stats
			where
				model_id = $1
			and
				hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(&mut *db)
	.await?;
	if let Some(row) = rows.get(0) {
		let data: String = row.get(0);
		let mut production_stats: ProductionStats = serde_json::from_str(&data)?;
		production_stats.update(model, monitor_event);
		let data = serde_json::to_string(&production_stats)?;
		sqlx::query(
			"
				update
					production_stats
				set
					data = $1
				where
					model_id = $2
				and
					hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_stats = ProductionStats::new(model, start_date, end_date);
		production_stats.update(model, monitor_event);
		let data = serde_json::to_string(&production_stats)?;
		sqlx::query(
			"
				insert into production_stats
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&data)
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}

async fn insert_or_update_production_metrics_for_monitor_event(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: tangram_model::ModelReader<'_>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string().to_string();
	let hour = monitor_event
		.date
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.with_nanosecond(0)
		.unwrap();
	let rows = sqlx::query(
		"
			select
				predictions.output
			from
				predictions
			where
				predictions.model_id = $1
			and
				predictions.identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_all(&mut *db)
	.await?;
	let row = rows
		.get(0)
		.ok_or_else(|| anyhow!("Failed to find prediction with identifier {}", identifier))?;
	let output: String = row.get(0);
	let output: PredictOutput = serde_json::from_str(&output)?;
	let prediction = match output {
		PredictOutput::Regression(RegressionPredictOutput { value }) => {
			NumberOrString::Number(value)
		}
		PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
		PredictOutput::MulticlassClassification(MulticlassClassificationPredictOutput {
			class_name,
			..
		}) => NumberOrString::String(class_name),
	};
	let true_value = match &monitor_event.true_value {
		serde_json::Value::Number(value) => {
			NumberOrString::Number(value.as_f64().unwrap().to_f32().unwrap())
		}
		serde_json::Value::String(value) => NumberOrString::String(value.clone()),
		_ => unimplemented!(),
	};
	let rows = sqlx::query(
		"
			select
				data
			from production_metrics
			where
				model_id = $1
			and
				hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(&mut *db)
	.await?;
	if let Some(row) = rows.get(0) {
		let data: String = row.get(0);
		let mut production_metrics: ProductionMetrics = serde_json::from_str(&data)?;
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_string(&production_metrics)?;
		sqlx::query(
			"
				update
					production_metrics
				set
					data = $1
				where
					model_id = $2
				and
					hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	} else {
		let start_date = hour;
		let end_date = hour + chrono::Duration::hours(1);
		let mut production_metrics = ProductionMetrics::new(model, start_date, end_date);
		production_metrics.update((prediction, true_value));
		let data = serde_json::to_string(&production_metrics)?;
		sqlx::query(
			"
				insert into production_metrics
					(model_id, data, hour)
				values
					($1, $2, $3)
			",
		)
		.bind(&model_id.to_string())
		.bind(&data)
		.bind(&hour.timestamp())
		.execute(&mut *db)
		.await?;
	}
	Ok(())
}
