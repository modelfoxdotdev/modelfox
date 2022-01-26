//! Functionality used to test core app components.

use crate::{
	alert::AlertMetric,
	monitor::{MonitorCadence, MonitorThreshold, MonitorThresholdMode},
	monitor_checker::MonitorConfig,
	App,
};
use anyhow::Result;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use num::ToPrimitive;
use std::{collections::HashMap, path::PathBuf};
use tangram_app_monitor_event::{
	BinaryClassificationPredictOutput, MonitorEvent, NumberOrString, PredictOutput,
	PredictionMonitorEvent, TrueValueMonitorEvent,
};
use tangram_id::Id;
use tangram_table::TableView;

const SQLITE_IN_MEMORY: &str = "sqlite::memory:";

pub fn init_test_options() -> crate::options::Options {
	let mut options = crate::options::Options::default();
	// set in-memory SQLite DB
	let database_url = SQLITE_IN_MEMORY.parse().expect("Malformed URL");
	let database_options = crate::options::DatabaseOptions {
		max_connections: None,
		url: database_url,
	};
	options.database = database_options;
	// Use in-memory storage
	options.storage = crate::options::StorageOptions::InMemory;
	options
}

pub async fn init_test_app() -> Result<App> {
	let options = init_test_options();
	let app = App::new(options).await?;
	Ok(app)
}

/// This module is nsested several layers deep, but the resources are relative to the workspace root
pub fn workspace_root() -> PathBuf {
	let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	crate_root
		.parent()
		.unwrap()
		.parent()
		.unwrap()
		.parent()
		.unwrap()
		.to_path_buf()
}

/// Add the Heart Disease model to a fresh repo, returning the model id
pub async fn init_heart_disease_model(app: &App) -> Result<Id> {
	let mut txn = app.begin_transaction().await?;
	let repo_id = app.create_root_repo(&mut txn, "Heart Disease").await?;
	let model_path = workspace_root().join("heart_disease.tangram");
	let model_id = app.add_model_to_repo(&mut txn, repo_id, model_path).await?;
	app.commit_transaction(txn).await?;
	Ok(model_id)
}

pub async fn seed_events(app: &App, examples_count: usize, model_id: Id) -> Result<()> {
	let target = "diagnosis";
	let class_names = Some(&["Negative", "Positive"]);
	let data_path = workspace_root().join("data").join("heart_disease.csv");
	let table = tangram_table::Table::from_path(&data_path, Default::default(), &mut |_| {})?;
	let mut idx = 0;
	let events: Vec<MonitorEvent> = (0..examples_count)
		.flat_map(|_| {
			let seed_float = idx as f32 / examples_count as f32;
			let id = Id::generate();
			let mut record = get_seeded_random_row(table.view(), seed_float);
			let target = record.remove(target).unwrap();
			// Rewrite asymptomatic to asx in 50% of rows.
			if idx < examples_count / 2 {
				let chest_pain = record.get_mut("chest_pain").unwrap();
				if chest_pain == "asymptomatic" {
					*chest_pain = serde_json::Value::String("asx".to_owned());
				}
			}
			let target_value = target.as_str().unwrap();
			let target_value = if seed_float > 0.6 {
				target_value
			} else {
				let class_names = class_names.unwrap();
				let random_target_value_index = (seed_float * class_names.len().to_f32().unwrap())
					.to_usize()
					.unwrap();
				class_names[random_target_value_index]
			};
			let output = PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
				class_name: target_value.to_string(),
				probability: 0.95,
			});
			let model_id = model_id.to_string();
			let date = get_seeded_random_date(seed_float);
			let mut events = vec![MonitorEvent::Prediction(PredictionMonitorEvent {
				date,
				identifier: NumberOrString::String(id.to_string()),
				input: record,
				model_id: model_id.parse().unwrap(),
				options: None,
				output,
			})];
			if idx as f32 / examples_count as f32 > 0.4 {
				events.push(MonitorEvent::TrueValue(TrueValueMonitorEvent {
					model_id: model_id.parse().unwrap(),
					identifier: NumberOrString::String(id.to_string()),
					true_value: target,
					date,
				}));
			}
			idx += 1;
			events
		})
		.collect();
	let mut txn = app.begin_transaction().await?;
	app.track_events(&mut txn, events).await?;
	app.commit_transaction(txn).await?;
	Ok(())
}

pub async fn seed_monitors(app: &App, model_id: Id) -> Result<()> {
	let mut txn = app.begin_transaction().await?;
	let monitor_configs = [
		MonitorConfig {
			cadence: MonitorCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.5),
				difference_upper: Some(0.5),
			},
			title: None,
		},
		MonitorConfig {
			cadence: MonitorCadence::Daily,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: Some(0.3),
				difference_upper: None,
			},
			title: None,
		},
		MonitorConfig {
			cadence: MonitorCadence::Weekly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.2),
				difference_upper: Some(0.1),
			},
			title: None,
		},
		MonitorConfig {
			cadence: MonitorCadence::Monthly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: None,
				difference_upper: Some(0.1),
			},
			title: None,
		},
	];
	for monitor_config in monitor_configs {
		app.create_monitor_from_config(&mut txn, model_id, &monitor_config)
			.await
			.unwrap();
	}
	app.commit_transaction(txn).await?;
	Ok(())
}

fn get_seeded_random_date(seed_float: f32) -> chrono::DateTime<Utc> {
	let end_time = chrono::Utc::now().timestamp();
	let start_time = chrono::Utc::now()
		.with_month(1)
		.unwrap()
		.with_day(1)
		.unwrap()
		.with_hour(0)
		.unwrap()
		.with_minute(0)
		.unwrap()
		.with_second(0)
		.unwrap()
		.timestamp();
	let time_range = (end_time - start_time).to_f32().unwrap();
	let timestamp = start_time + (seed_float * time_range.trunc()).to_i64().unwrap();
	Utc.timestamp(timestamp, 0)
}

fn get_seeded_random_row(table: TableView, seed_float: f32) -> HashMap<String, serde_json::Value> {
	let random_row_index = (table.nrows().to_f32().unwrap() * seed_float)
		.to_usize()
		.unwrap();
	table
		.columns()
		.iter()
		.map(|column| match column {
			tangram_table::TableColumnView::Number(column) => {
				let column_name = column.name().unwrap().to_owned();
				let value = column.data()[random_row_index].to_f64().unwrap();
				let value = if let Some(value) = serde_json::Number::from_f64(value) {
					serde_json::Value::Number(value)
				} else {
					serde_json::Value::Null
				};
				(column_name, value)
			}
			tangram_table::TableColumnView::Enum(column) => {
				let column_name = column.name().unwrap().to_owned();
				let value = column.data()[random_row_index];
				let value = match value {
					Some(value) => serde_json::Value::String(
						column.variants().get(value.get() - 1).unwrap().clone(),
					),
					None => serde_json::Value::Null,
				};
				(column_name, value)
			}
			_ => unimplemented!(),
		})
		.collect::<HashMap<String, serde_json::Value>>()
}
