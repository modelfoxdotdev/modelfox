use anyhow::{bail, Result};
use clap::{ArgEnum, Parser};
use num::ToPrimitive;
use rand::Rng;
use std::{collections::HashMap, path::Path};
use tangram_app_core::{
	alerts::{AlertCadence, AlertMetric, MonitorThreshold, MonitorThresholdMode},
	monitor::MonitorConfig,
	monitor_event::{
		BinaryClassificationPredictOutput, MonitorEvent, MulticlassClassificationPredictOutput,
		NumberOrString, PredictOutput, PredictionMonitorEvent, RegressionPredictOutput,
		TrueValueMonitorEvent,
	},
	reset_data, App,
};
use tangram_id::Id;
use tangram_table::TableView;
use url::Url;

#[derive(Parser)]
pub struct Args {
	#[clap(long)]
	pub database_url: Option<Url>,
	#[clap(long, default_value = "1000")]
	pub examples_count: usize,
}

#[derive(Clone, ArgEnum)]
pub enum Dataset {
	#[clap(name = "heart_disease")]
	HeartDisease,
}

struct DatasetConfig {
	path: &'static str,
	model_path: &'static str,
	name: &'static str,
	target: &'static str,
	class_names: Option<&'static [&'static str]>,
}

const HEART_DISEASE: DatasetConfig = DatasetConfig {
	path: "data/heart_disease.csv",
	model_path: "heart_disease.tangram",
	name: "heart_disease",
	target: "diagnosis",
	class_names: Some(&["Negative", "Positive"]),
};

#[tokio::main]
pub async fn main() -> Result<()> {
	let args = Args::parse();
	let dataset = HEART_DISEASE;
	let table =
		tangram_table::Table::from_path(Path::new(dataset.path), Default::default(), &mut |_| {})?;
	let mut rng = rand::thread_rng();
	reset_data(&args.database_url).await?;
	let options = init_options(args.database_url)?;
	let app = App::new(options).await?;
	let mut txn = app.begin_transaction().await?;
	let repo_id = app.create_root_repo(&mut txn, "Heart Disease").await?;
	let model_id = app
		.add_model_to_repo(&mut txn, repo_id, dataset.model_path)
		.await?;
	let events: Vec<MonitorEvent> = (0..args.examples_count)
		.flat_map(|_| {
			let id = Id::generate();
			let mut record = get_random_row(table.view());
			let target = record.remove(dataset.target).unwrap();
			if dataset.name == "heart_disease" {
				// Rewrite asymptomatic to asx in 50% of rows.
				if rng.gen::<bool>() {
					let chest_pain = record.get_mut("chest_pain").unwrap();
					if chest_pain == "asymptomatic" {
						*chest_pain = serde_json::Value::String("asx".to_owned());
					}
				}
			}
			let output = generate_fake_prediction(&target, &dataset);
			let model_id = model_id.to_string();
			let date = get_random_date();
			let mut events = vec![MonitorEvent::Prediction(PredictionMonitorEvent {
				date,
				identifier: NumberOrString::String(id.to_string()),
				input: record,
				model_id: model_id.parse().unwrap(),
				options: None,
				output,
			})];
			if rng.gen::<f32>() > 0.4 {
				events.push(MonitorEvent::TrueValue(TrueValueMonitorEvent {
					model_id: model_id.parse().unwrap(),
					identifier: NumberOrString::String(id.to_string()),
					true_value: target,
					date,
				}));
			}
			events
		})
		.collect();
	let monitors = generate_fake_monitors();
	for config in monitors {
		app.create_monitor_from_config(&mut txn, model_id, &config)
			.await?;
	}
	app.track_events(&mut txn, events).await?;
	app.commit_transaction(txn).await?;
	Ok(())
}

fn init_options(database_url: Option<Url>) -> Result<tangram_app_core::options::Options> {
	let mut options = tangram_app_core::options::Options::default();
	if database_url.is_none() {
		return Ok(options);
	}

	let database_url = database_url.unwrap();
	match database_url.scheme() {
		"postgres" | "sqlite" => {
			let database_options = tangram_app_core::options::DatabaseOptions {
				max_connections: None,
				url: database_url,
			};
			options.database = database_options;
			Ok(options)
		}
		_ => bail!("Unsupported database URL scheme"),
	}
}

fn generate_fake_monitors() -> Vec<MonitorConfig> {
	vec![
		MonitorConfig {
			cadence: AlertCadence::Hourly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.5),
				difference_upper: Some(0.5),
			},
			title: None,
		},
		MonitorConfig {
			cadence: AlertCadence::Daily,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: Some(0.3),
				difference_upper: None,
			},
			title: None,
		},
		MonitorConfig {
			cadence: AlertCadence::Weekly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Absolute,
				difference_lower: Some(0.2),
				difference_upper: Some(0.1),
			},
			title: None,
		},
		MonitorConfig {
			cadence: AlertCadence::Monthly,
			threshold: MonitorThreshold {
				metric: AlertMetric::Accuracy,
				mode: MonitorThresholdMode::Percentage,
				difference_lower: None,
				difference_upper: Some(0.1),
			},
			title: None,
		},
	]
}

fn generate_fake_prediction(target: &serde_json::Value, dataset: &DatasetConfig) -> PredictOutput {
	match dataset.name {
		"heart_disease" => generate_fake_prediction_heart_disease(target, dataset),
		"iris" => generate_fake_prediction_iris(target, dataset),
		"boston" => generate_fake_prediction_boston(target),
		_ => unimplemented!(),
	}
}

fn generate_fake_prediction_boston(target_value: &serde_json::Value) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_f64().unwrap();
	let target_value = target_value + rng.gen::<f64>() * 5.0;
	PredictOutput::Regression(RegressionPredictOutput {
		value: target_value.to_f32().unwrap(),
	})
}

fn generate_fake_prediction_heart_disease(
	target_value: &serde_json::Value,
	dataset: &DatasetConfig,
) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_str().unwrap();
	let target_value = if rng.gen::<f32>() > 0.6 {
		target_value
	} else {
		let class_names = dataset.class_names.unwrap();
		let random_target_value_index = (rng.gen::<f32>() * class_names.len().to_f32().unwrap())
			.to_usize()
			.unwrap();
		class_names[random_target_value_index]
	};
	PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
		class_name: target_value.to_string(),
		probability: 0.95,
	})
}

fn generate_fake_prediction_iris(
	target_value: &serde_json::Value,
	dataset: &DatasetConfig,
) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_str().unwrap();
	let target_value = if rng.gen::<f32>() > 0.6 {
		target_value
	} else {
		let class_names = dataset.class_names.unwrap();
		let random_target_value_index = (rng.gen::<f32>() * class_names.len().to_f32().unwrap())
			.to_usize()
			.unwrap();
		class_names[random_target_value_index]
	};
	let probabilities = dataset
		.class_names
		.unwrap()
		.iter()
		.map(|class_name| {
			if class_name == &target_value {
				(class_name.to_string(), 0.95)
			} else {
				(class_name.to_string(), 0.025)
			}
		})
		.collect::<HashMap<String, f32>>();
	PredictOutput::MulticlassClassification(MulticlassClassificationPredictOutput {
		class_name: target_value.to_string(),
		probabilities,
	})
}

fn get_random_date() -> time::OffsetDateTime {
	let end_time = time::OffsetDateTime::now_utc().timestamp();
	let start_time = time::OffsetDateTime::now_utc()
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
	let timestamp = start_time
		+ (rand::thread_rng().gen::<f32>() * time_range.trunc())
			.to_i64()
			.unwrap();
	Utc.timestamp(timestamp, 0)
}

fn get_random_row(table: TableView) -> HashMap<String, serde_json::Value> {
	let mut rng = rand::thread_rng();
	let random_row_index = (table.nrows().to_f32().unwrap() * rng.gen::<f32>())
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
