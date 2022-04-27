use crate::{model::get_model_bytes, storage::Storage};
use anyhow::{anyhow, bail, Result};
use chrono::prelude::*;
use memmap::Mmap;
use modelfox_app_monitor_event::{
	BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
	NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
	TrueValueMonitorEvent,
};
use modelfox_app_production_metrics::ProductionMetrics;
use modelfox_app_production_stats::ProductionStats;
use modelfox_id::Id;
use num::ToPrimitive;
use sqlx::prelude::*;
use std::{borrow::BorrowMut, collections::BTreeMap};
use tracing::error;

use super::App;

impl App {
	pub async fn track_events(
		&self,
		txn: &mut sqlx::Transaction<'_, sqlx::Any>,
		events: Vec<MonitorEvent>,
	) -> Result<()> {
		let mut txn = txn.begin().await?;
		let mut model_cache = BTreeMap::new();
		for event in events {
			match event {
				MonitorEvent::Prediction(monitor_event) => {
					let handle_prediction_result = handle_prediction_monitor_event(
						&mut txn,
						&self.state.storage,
						&mut model_cache,
						monitor_event,
					)
					.await;
					if let Err(e) = handle_prediction_result {
						error!(%e);
						return Err(anyhow!("{}", e));
					}
				}
				MonitorEvent::TrueValue(monitor_event) => {
					let handle_true_value_result = handle_true_value_monitor_event(
						&mut txn,
						&self.state.storage,
						&mut model_cache,
						monitor_event,
					)
					.await;
					if handle_true_value_result.is_err() {
						return Err(anyhow!(
							"{}",
							handle_true_value_result.err().unwrap().to_string()
						));
					}
				}
			}
		}
		txn.commit().await?;
		Ok(())
	}
}

pub async fn handle_prediction_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
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
	let model = modelfox_model::from_bytes(bytes)?;
	write_prediction_monitor_event(txn, model_id, &monitor_event).await?;
	insert_or_update_production_stats_for_monitor_event(txn, model_id, model, monitor_event)
		.await?;
	Ok(())
}

pub async fn write_prediction_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &PredictionMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string();
	let row = sqlx::query(
		"
			select count(*) from predictions
			where
				model_id = $1
				and identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier.to_string())
	.fetch_one(txn.borrow_mut())
	.await?;
	let prediction_count: i64 = row.get(0);
	if prediction_count > 0 {
		bail!("A prediction has already been logged with this identifier.");
	}
	let prediction_monitor_event_id = Id::generate();
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
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

pub async fn handle_true_value_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
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
	let model = modelfox_model::from_bytes(bytes)?;
	write_true_value_monitor_event(txn, model_id, &monitor_event).await?;
	insert_or_update_production_metrics_for_monitor_event(txn, model_id, model, monitor_event)
		.await?;
	Ok(())
}

pub async fn write_true_value_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	monitor_event: &TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string();
	let row = sqlx::query(
		"
			select count(*) from true_values
			where
				model_id = $1
				and identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier.to_string())
	.fetch_one(txn.borrow_mut())
	.await?;
	let true_value_count: i64 = row.get(0);
	if true_value_count > 0 {
		bail!("A prediction has already been logged with this identifier.");
	}
	let true_value_monitor_event_id = Id::generate();
	let date = monitor_event.date;
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
	.execute(txn.borrow_mut())
	.await?;
	Ok(())
}

pub async fn insert_or_update_production_stats_for_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: modelfox_model::ModelReader<'_>,
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
				and hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_all(txn.borrow_mut())
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
					and hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(txn.borrow_mut())
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
		.execute(txn.borrow_mut())
		.await?;
	}
	Ok(())
}

pub async fn insert_or_update_production_metrics_for_monitor_event(
	txn: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
	model: modelfox_model::ModelReader<'_>,
	monitor_event: TrueValueMonitorEvent,
) -> Result<()> {
	let identifier = monitor_event.identifier.as_string().to_string();
	let rows = sqlx::query(
		"
			select
				output,
				date
			from
				predictions
			where
				predictions.model_id = $1
				and predictions.identifier = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&identifier)
	.fetch_all(txn.borrow_mut())
	.await?;
	if rows.is_empty() {
		bail!("Failed to find prediction with identifier {}", identifier);
	}
	let true_value = match &monitor_event.true_value {
		serde_json::Value::Number(value) => {
			NumberOrString::Number(value.as_f64().unwrap().to_f32().unwrap())
		}
		serde_json::Value::String(value) => NumberOrString::String(value.clone()),
		_ => unimplemented!(),
	};
	let row = rows
		.get(0)
		.ok_or_else(|| anyhow!("Failed to find prediction with identifier {}", identifier))?;
	let output: String = row.get(0);
	let date: i64 = row.get(1);
	let date = Utc.timestamp(date, 0);
	let hour = date
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.with_nanosecond(0)
		.unwrap();
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
	let row = sqlx::query(
		"
			select
				data
			from production_metrics
			where
				model_id = $1
				and hour = $2
		",
	)
	.bind(&model_id.to_string())
	.bind(&hour.timestamp())
	.fetch_optional(txn.borrow_mut())
	.await?;
	if let Some(row) = row {
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
					and hour = $3
			",
		)
		.bind(&data)
		.bind(&model_id.to_string())
		.bind(&hour.timestamp())
		.execute(txn.borrow_mut())
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
		.execute(txn.borrow_mut())
		.await?;
	}
	Ok(())
}
