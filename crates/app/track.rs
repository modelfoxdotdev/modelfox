use anyhow::Result;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use clap::Parser;
use num::ToPrimitive;
use rand::Rng;
use std::{collections::HashMap, path::Path};
use tangram_app_core::monitor_event::{
	BinaryClassificationPredictOutput, MonitorEvent, NumberOrString, PredictOutput,
	PredictionMonitorEvent, TrueValueMonitorEvent,
};
use tangram_id::Id;
use tangram_table::TableView;
use url::Url;

#[derive(Parser)]
pub struct Args {
	#[clap(long)]
	pub tangram_url: Url,
	#[clap(long)]
	path: String,
	#[clap(long)]
	target: String,
	#[clap(long)]
	class_names: Option<Vec<String>>,
	#[clap(long)]
	pub model_id: String,
	#[clap(long)]
	pub examples_count: usize,
}

pub fn main() -> Result<()> {
	let args = Args::parse();
	let table =
		tangram_table::Table::from_path(Path::new(&args.path), Default::default(), &mut |_| {})?;
	let mut rng = rand::thread_rng();
	let events: Vec<MonitorEvent> = (0..args.examples_count)
		.flat_map(|_| {
			let id = Id::generate();
			let mut record = get_random_row(table.view());
			let target = record.remove(&args.target).unwrap();
			let output =
				generate_fake_prediction(&target, &args.path, &args.target, &args.class_names);
			let model_id: &str = args.model_id.as_str();
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
	for events in events.chunks(100) {
		track_events(&args.tangram_url, events);
	}
	Ok(())
}

fn generate_fake_prediction(
	target_value: &serde_json::Value,
	_path: &str,
	_target: &str,
	class_names: &Option<Vec<String>>,
) -> PredictOutput {
	let mut rng = rand::thread_rng();
	let target_value = target_value.as_str().unwrap();
	let target_value = if rng.gen::<f32>() > 0.6 {
		target_value.to_owned()
	} else {
		let class_names = class_names.as_ref().unwrap();
		let random_target_value_index = (rng.gen::<f32>() * class_names.len().to_f32().unwrap())
			.to_usize()
			.unwrap();
		class_names[random_target_value_index].to_owned()
	};
	PredictOutput::BinaryClassification(BinaryClassificationPredictOutput {
		class_name: target_value,
		probability: 0.95,
	})
}

fn get_random_date() -> chrono::DateTime<Utc> {
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
			tangram_table::TableColumnView::Text(column) => {
				let column_name = column.name().unwrap().to_owned();
				let value = &column.data()[random_row_index];
				let value = serde_json::Value::String(value.to_string());
				(column_name, value)
			}
			_ => unimplemented!(),
		})
		.collect::<HashMap<String, serde_json::Value>>()
}

fn track_events(tangram_url: &Url, events: &[MonitorEvent]) {
	let client = reqwest::blocking::Client::new();
	let mut url = tangram_url.clone();
	url.set_path("/track");
	client.post(url).json(&events).send().unwrap();
}
