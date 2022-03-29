use modelfox_finite::NotFiniteError;
use std::{borrow::Cow, collections::HashMap};
use modelfox_id::Id;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum MonitorEvent {
	#[serde(rename = "prediction")]
	Prediction(PredictionMonitorEvent),
	#[serde(rename = "true_value", alias = "trueValue")]
	TrueValue(TrueValueMonitorEvent),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PredictionMonitorEvent {
	#[serde(alias = "modelId")]
	pub model_id: Id,
	pub date: chrono::DateTime<chrono::Utc>,
	pub identifier: NumberOrString,
	pub options: Option<PredictOptions>,
	pub input: HashMap<String, serde_json::Value>,
	pub output: PredictOutput,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PredictOptions {
	pub threshold: f32,
	#[serde(alias = "computeFeatureContributions")]
	pub compute_feature_contributions: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TrueValueMonitorEvent {
	#[serde(alias = "modelId")]
	pub model_id: Id,
	pub date: chrono::DateTime<chrono::Utc>,
	pub identifier: NumberOrString,
	#[serde(alias = "trueValue")]
	pub true_value: serde_json::Value,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RegressionPredictOutput {
	pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BinaryClassificationPredictOutput {
	#[serde(alias = "className")]
	pub class_name: String,
	pub probability: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MulticlassClassificationPredictOutput {
	#[serde(alias = "className")]
	pub class_name: String,
	pub probabilities: HashMap<String, f32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum NumberOrString {
	Number(f32),
	String(String),
}

impl NumberOrString {
	pub fn as_number(&self) -> Result<f32, NotFiniteError> {
		match self {
			NumberOrString::Number(number) => Ok(*number),
			NumberOrString::String(string) => match string.parse() {
				Ok(value) => Ok(value),
				Err(_) => Err(NotFiniteError),
			},
		}
	}
	pub fn as_string(&self) -> Cow<str> {
		match self {
			NumberOrString::Number(number) => number.to_string().into(),
			NumberOrString::String(string) => string.into(),
		}
	}
}
