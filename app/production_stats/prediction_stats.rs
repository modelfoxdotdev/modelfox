use super::number_stats::{NumberStats, NumberStatsOutput};
use std::collections::BTreeMap;
use tangram_app_common::monitor_event::PredictOutput;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ProductionPredictionStats {
	#[serde(rename = "regression")]
	Regression(RegressionProductionPredictionStats),
	#[serde(rename = "binary_classification")]
	BinaryClassification(ClassificationProductionPredictionStats),
	#[serde(rename = "multiclass_classification")]
	MulticlassClassification(ClassificationProductionPredictionStats),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct RegressionProductionPredictionStats {
	stats: Option<NumberStats>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ClassificationProductionPredictionStats {
	pub histogram: BTreeMap<String, u64>,
}

#[derive(Debug)]
pub enum ProductionPredictionStatsOutput {
	Regression(RegressionProductionPredictionStatsOutput),
	BinaryClassification(ClassificationProductionPredictionStatsOutput),
	MulticlassClassification(ClassificationProductionPredictionStatsOutput),
}

#[derive(Debug)]
pub struct RegressionProductionPredictionStatsOutput {
	pub stats: Option<NumberStatsOutput>,
}

#[derive(Debug)]
pub struct ClassificationProductionPredictionStatsOutput {
	pub histogram: Vec<(String, u64)>,
}

impl ProductionPredictionStats {
	pub fn new(model: tangram_model::ModelReader) -> ProductionPredictionStats {
		match model.inner() {
			tangram_model::ModelInnerReader::Regressor(_) => {
				ProductionPredictionStats::Regression(RegressionProductionPredictionStats::new())
			}
			tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
				let binary_classifier = binary_classifier.read();
				ProductionPredictionStats::BinaryClassification(
					ClassificationProductionPredictionStats::new(vec![
						binary_classifier.negative_class().to_owned(),
						binary_classifier.positive_class().to_owned(),
					]),
				)
			}
			tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
				let multiclass_classifier = multiclass_classifier.read();
				ProductionPredictionStats::MulticlassClassification(
					ClassificationProductionPredictionStats::new(
						multiclass_classifier
							.classes()
							.iter()
							.map(ToOwned::to_owned)
							.collect(),
					),
				)
			}
		}
	}

	pub fn update(&mut self, value: PredictOutput) {
		match self {
			ProductionPredictionStats::Regression(stats) => stats.update(value),
			ProductionPredictionStats::BinaryClassification(stats) => stats.update(value),
			ProductionPredictionStats::MulticlassClassification(stats) => stats.update(value),
		}
	}

	pub fn merge(&mut self, other: ProductionPredictionStats) {
		match self {
			ProductionPredictionStats::Regression(this) => {
				if let ProductionPredictionStats::Regression(other) = other {
					this.merge(other)
				}
			}
			ProductionPredictionStats::BinaryClassification(this) => {
				if let ProductionPredictionStats::BinaryClassification(other) = other {
					this.merge(other)
				}
			}
			ProductionPredictionStats::MulticlassClassification(this) => {
				if let ProductionPredictionStats::MulticlassClassification(other) = other {
					this.merge(other)
				}
			}
		}
	}

	pub fn finalize(self) -> ProductionPredictionStatsOutput {
		match self {
			ProductionPredictionStats::Regression(stats) => {
				ProductionPredictionStatsOutput::Regression(stats.finalize())
			}
			ProductionPredictionStats::BinaryClassification(stats) => {
				ProductionPredictionStatsOutput::BinaryClassification(stats.finalize())
			}
			ProductionPredictionStats::MulticlassClassification(stats) => {
				ProductionPredictionStatsOutput::MulticlassClassification(stats.finalize())
			}
		}
	}
}

impl RegressionProductionPredictionStats {
	fn new() -> RegressionProductionPredictionStats {
		RegressionProductionPredictionStats { stats: None }
	}

	pub fn update(&mut self, value: PredictOutput) {
		let value = match value {
			PredictOutput::Regression(value) => value,
			_ => unreachable!(),
		};
		match &mut self.stats {
			None => {
				self.stats.replace(NumberStats::new(value.value));
			}
			Some(stats) => stats.update(value.value),
		};
	}

	pub fn merge(&mut self, other: RegressionProductionPredictionStats) {
		match &mut self.stats {
			None => self.stats = other.stats,
			Some(stats) => {
				if let Some(other) = other.stats {
					stats.merge(other)
				}
			}
		};
	}

	pub fn finalize(self) -> RegressionProductionPredictionStatsOutput {
		let stats = self.stats.map(|s| s.finalize());
		RegressionProductionPredictionStatsOutput { stats }
	}
}

impl ClassificationProductionPredictionStats {
	pub fn new(classes: Vec<String>) -> ClassificationProductionPredictionStats {
		let histogram = classes.into_iter().map(|class| (class, 0)).collect();
		ClassificationProductionPredictionStats { histogram }
	}

	pub fn update(&mut self, value: PredictOutput) {
		let class_name = match value {
			PredictOutput::BinaryClassification(value) => value.class_name,
			PredictOutput::MulticlassClassification(value) => value.class_name,
			_ => unreachable!(),
		};
		if let Some(count) = self.histogram.get_mut(&class_name) {
			*count += 1;
		}
	}

	pub fn merge(&mut self, other: ClassificationProductionPredictionStats) {
		for (value, count) in other.histogram.into_iter() {
			*self.histogram.entry(value).or_insert(0) += count;
		}
	}

	pub fn finalize(self) -> ClassificationProductionPredictionStatsOutput {
		ClassificationProductionPredictionStatsOutput {
			histogram: self.histogram.into_iter().collect(),
		}
	}
}
