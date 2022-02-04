//! Functionality used to test core app components.

use crate::{
	alert::{AlertMethod, AlertMetric},
	model::get_model_bytes,
	monitor::{MonitorCadence, MonitorThreshold, MonitorThresholdMode},
	monitor_checker::MonitorConfig,
	App,
};
use anyhow::{bail, Result};
use chrono::{Datelike, TimeZone, Timelike, Utc};
use num::ToPrimitive;
use std::{collections::HashMap, path::PathBuf};
use tangram::ClassificationOutputValue;
use tangram_app_monitor_event::{
	BinaryClassificationPredictOutput, MonitorEvent, NumberOrString, PredictOutput,
	PredictionMonitorEvent, TrueValueMonitorEvent,
};
use tangram_id::Id;
use tangram_table::TableView;

pub fn init_test_options() -> crate::options::Options {
	let mut options = crate::options::Options::default();
	// set in-memory SQLite DB
	let database_url = "sqlite::memory:".parse().expect("Malformed URL");
	let database_options = crate::options::DatabaseOptions {
		max_connections: Some(10),
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
	let data_path = workspace_root().join("heart_disease.csv");
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
			if seed_float > 0.4 {
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

#[derive(Clone, Debug, tangram::PredictInput)]
pub struct Input {
	pub age: f32,
	pub gender: Gender,
	pub chest_pain: ChestPain,
	pub resting_blood_pressure: f32,
	pub cholesterol: f32,
	pub fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120,
	pub resting_ecg_result: RestingEcgResult,
	pub exercise_max_heart_rate: f32,
	pub exercise_induced_angina: ExerciseInducedAngina,
	pub exercise_st_depression: f32,
	pub exercise_st_slope: ExerciseStSlope,
	pub fluoroscopy_vessels_colored: FluoroscopyVesselsColored,
	pub thallium_stress_test: ThalliumStressTest,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum Gender {
	#[tangram(value = "male")]
	Male,
	#[tangram(value = "female")]
	Female,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ChestPain {
	#[tangram(value = "asymptomatic")]
	Asymptomatic,
	#[tangram(value = "non-angina pain")]
	NonAnginaPain,
	#[tangram(value = "atypical angina")]
	AtypicalAngina,
	#[tangram(value = "typical angina")]
	TypicalAngina,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum FastingBloodSugarGreaterThan120 {
	#[tangram(value = "false")]
	False,
	#[tangram(value = "true")]
	True,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum RestingEcgResult {
	#[tangram(value = "normal")]
	Normal,
	#[tangram(value = "probable or definite left ventricular hypertrophy")]
	Lvh,
	#[tangram(value = "ST-T wave abnormality")]
	SttWaveAbnormality,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ExerciseInducedAngina {
	#[tangram(value = "no")]
	No,
	#[tangram(value = "yes")]
	Yes,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ExerciseStSlope {
	#[tangram(value = "upsloping")]
	Upsloping,
	#[tangram(value = "flat")]
	Flat,
	#[tangram(value = "downsloping")]
	Downsloping,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum FluoroscopyVesselsColored {
	#[tangram(value = "0")]
	Zero,
	#[tangram(value = "1")]
	One,
	#[tangram(value = "2")]
	Two,
	#[tangram(value = "3")]
	Three,
}

#[derive(Clone, Debug, tangram::PredictInputValue)]
pub enum ThalliumStressTest {
	#[tangram(value = "normal")]
	Normal,
	#[tangram(value = "reversible defect")]
	ReversibleDefect,
	#[tangram(value = "fixed defect")]
	FixedDefect,
}

type Output = tangram::BinaryClassificationPredictOutput<Diagnosis>;

#[derive(Clone, Debug, tangram::ClassificationOutputValue)]
enum Diagnosis {
	#[tangram(value = "Negative")]
	Negative,
	#[tangram(value = "Positive")]
	Positive,
}
/// Returns the ID of the prediction as well as the predicted value
pub async fn seed_single_prediction_event(app: &App, model_id: Id) -> Result<(Id, String)> {
	let data_path = workspace_root().join("heart_disease.csv");
	let table = tangram_table::Table::from_path(&data_path, Default::default(), &mut |_| {})?;

	let id = Id::generate();
	let identifier = NumberOrString::String(id.to_string());

	// NOTE - we use the app's clock for the date, but this code still requires chrono
	// Fix this when we switch entirely
	let now = app.clock().now_utc().unix_timestamp();
	let naive_date = chrono::NaiveDateTime::from_timestamp(now, 0);
	let date = chrono::DateTime::from_utc(naive_date, chrono::Utc);

	let record = get_seeded_random_row(table.view(), 0.5);

	let input = Input {
		age: 63.0,
		gender: Gender::Male,
		chest_pain: ChestPain::TypicalAngina,
		resting_blood_pressure: 145.0,
		cholesterol: 233.0,
		fasting_blood_sugar_greater_than_120: FastingBloodSugarGreaterThan120::True,
		resting_ecg_result: RestingEcgResult::Lvh,
		exercise_max_heart_rate: 150.0,
		exercise_induced_angina: ExerciseInducedAngina::No,
		exercise_st_depression: 2.3,
		exercise_st_slope: ExerciseStSlope::Downsloping,
		fluoroscopy_vessels_colored: FluoroscopyVesselsColored::Zero,
		thallium_stress_test: ThalliumStressTest::FixedDefect,
	};

	// Ask the model to predict for the input
	let bytes = get_model_bytes(app.storage(), model_id).await?;
	let model = tangram::Model::<Input, Output>::from_bytes(&bytes, None)?;
	let output = model.predict_one(input.clone(), None);
	let output_class_name = output.class_name.as_str().to_owned();
	let output = PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
		class_name: output_class_name.clone(),
		probability: output.probability,
	});

	let result = MonitorEvent::Prediction(PredictionMonitorEvent {
		date,
		identifier,
		input: record,
		model_id,
		options: None,
		output,
	});

	let mut txn = app.begin_transaction().await?;
	app.track_events(&mut txn, vec![result]).await?;
	app.commit_transaction(txn).await?;

	Ok((id, output_class_name))
}

pub async fn seed_single_true_value_event(
	app: &App,
	model_id: Id,
	id: Id,
	prediction_result: String,
	correct_prediction: bool,
) -> Result<()> {
	// NOTE - we use the app's clock for the date, but this code still requires chrono
	// Fix this when we switch entirely
	let now = app.clock().now_utc().unix_timestamp();
	let naive_date = chrono::NaiveDateTime::from_timestamp(now, 0);
	let date = chrono::DateTime::from_utc(naive_date, chrono::Utc);

	let target = if correct_prediction {
		match prediction_result.as_str() {
			"Positive" => "Positive",
			"Negative" => "Negative",
			_ => bail!("Unexpected prediction result!"),
		}
	} else {
		match prediction_result.as_str() {
			"Positive" => "Negative",
			"Negative" => "Positive",
			_ => bail!("Unexpected prediction result!"),
		}
	};

	let result = MonitorEvent::TrueValue(TrueValueMonitorEvent {
		model_id,
		identifier: NumberOrString::String(id.to_string()),
		true_value: target.into(),
		date,
	});

	let mut txn = app.begin_transaction().await?;
	app.track_events(&mut txn, vec![result]).await?;
	app.commit_transaction(txn).await?;

	Ok(())
}

/// Seeds a prediction and a true value at once
pub async fn seed_monitor_event_pair(
	app: &App,
	model_id: Id,
	correct_prediction: bool,
) -> Result<()> {
	let (prediction_id, prediction_result) = seed_single_prediction_event(app, model_id).await?;
	seed_single_true_value_event(
		app,
		model_id,
		prediction_id,
		prediction_result,
		correct_prediction,
	)
	.await?;
	Ok(())
}

pub async fn seed_single_monitor(
	app: &App,
	monitor_config: &MonitorConfig,
	model_id: Id,
) -> Result<()> {
	let mut txn = app.begin_transaction().await?;
	app.create_monitor_from_config(&mut txn, model_id, monitor_config)
		.await?;
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
				difference_lower: Some(0.1),
				difference_upper: Some(0.1),
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
		},
		MonitorConfig {
			cadence: MonitorCadence::Daily,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: Some(20.0),
				difference_upper: None,
			},
			title: None,
			methods: vec![AlertMethod::Stdout],
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
			methods: vec![AlertMethod::Stdout],
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
			methods: vec![AlertMethod::Stdout],
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
