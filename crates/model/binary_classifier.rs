use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BinaryClassifier {
	#[tangram_serialize(id = 0, required)]
	pub target_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub negative_class: String,
	#[tangram_serialize(id = 2, required)]
	pub positive_class: String,
	#[tangram_serialize(id = 3, required)]
	pub train_row_count: u64,
	#[tangram_serialize(id = 4, required)]
	pub test_row_count: u64,
	#[tangram_serialize(id = 5, required)]
	pub overall_row_count: u64,
	#[tangram_serialize(id = 6, required)]
	pub stats_settings: StatsSettings,
	#[tangram_serialize(id = 7, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 8, required)]
	pub overall_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 9, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 10, required)]
	pub train_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 11, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 12, required)]
	pub test_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 13, required)]
	pub baseline_metrics: BinaryClassificationMetrics,
	#[tangram_serialize(id = 14, required)]
	pub comparison_metric: BinaryClassificationComparisonMetric,
	#[tangram_serialize(id = 15, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[tangram_serialize(id = 16, required)]
	pub best_grid_item_index: u64,
	#[tangram_serialize(id = 17, required)]
	pub model: BinaryClassificationModel,
	#[tangram_serialize(id = 18, required)]
	pub test_metrics: BinaryClassificationMetrics,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum BinaryClassificationComparisonMetric {
	#[tangram_serialize(id = 0)]
	Aucroc,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BinaryClassificationMetrics {
	#[tangram_serialize(id = 0, required)]
	pub auc_roc: f32,
	#[tangram_serialize(id = 1, required)]
	pub default_threshold: BinaryClassificationMetricsForThreshold,
	#[tangram_serialize(id = 2, required)]
	pub thresholds: Vec<BinaryClassificationMetricsForThreshold>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BinaryClassificationMetricsForThreshold {
	#[tangram_serialize(id = 0, required)]
	pub threshold: f32,
	#[tangram_serialize(id = 1, required)]
	pub true_positives: u64,
	#[tangram_serialize(id = 2, required)]
	pub false_positives: u64,
	#[tangram_serialize(id = 3, required)]
	pub true_negatives: u64,
	#[tangram_serialize(id = 4, required)]
	pub false_negatives: u64,
	#[tangram_serialize(id = 5, required)]
	pub accuracy: f32,
	#[tangram_serialize(id = 6, required)]
	pub precision: Option<f32>,
	#[tangram_serialize(id = 7, required)]
	pub recall: Option<f32>,
	#[tangram_serialize(id = 8, required)]
	pub f1_score: Option<f32>,
	#[tangram_serialize(id = 9, required)]
	pub true_positive_rate: f32,
	#[tangram_serialize(id = 10, required)]
	pub false_positive_rate: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum BinaryClassificationModel {
	#[tangram_serialize(id = 0)]
	Linear(LinearBinaryClassifier),
	#[tangram_serialize(id = 1)]
	Tree(TreeBinaryClassifier),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct LinearBinaryClassifier {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_linear::serialize::BinaryClassifier,
	#[tangram_serialize(id = 1, required)]
	pub train_options: LinearModelTrainOptions,
	#[tangram_serialize(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[tangram_serialize(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[tangram_serialize(id = 4, required)]
	pub feature_importances: Vec<f32>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TreeBinaryClassifier {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_tree::serialize::BinaryClassifier,
	#[tangram_serialize(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[tangram_serialize(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[tangram_serialize(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[tangram_serialize(id = 4, required)]
	pub feature_importances: Vec<f32>,
}
