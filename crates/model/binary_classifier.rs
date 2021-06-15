use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BinaryClassifier {
	#[buffalo(id = 0, required)]
	pub target_column_name: String,
	#[buffalo(id = 1, required)]
	pub negative_class: String,
	#[buffalo(id = 2, required)]
	pub positive_class: String,
	#[buffalo(id = 3, required)]
	pub train_row_count: u64,
	#[buffalo(id = 4, required)]
	pub test_row_count: u64,
	#[buffalo(id = 5, required)]
	pub overall_row_count: u64,
	#[buffalo(id = 6, required)]
	pub stats_settings: StatsSettings,
	#[buffalo(id = 7, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 8, required)]
	pub overall_target_column_stats: ColumnStats,
	#[buffalo(id = 9, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 10, required)]
	pub train_target_column_stats: ColumnStats,
	#[buffalo(id = 11, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 12, required)]
	pub test_target_column_stats: ColumnStats,
	#[buffalo(id = 13, required)]
	pub baseline_metrics: BinaryClassificationMetrics,
	#[buffalo(id = 14, required)]
	pub comparison_metric: BinaryClassificationComparisonMetric,
	#[buffalo(id = 15, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[buffalo(id = 16, required)]
	pub best_grid_item_index: u64,
	#[buffalo(id = 17, required)]
	pub model: BinaryClassificationModel,
	#[buffalo(id = 18, required)]
	pub test_metrics: BinaryClassificationMetrics,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum BinaryClassificationComparisonMetric {
	#[buffalo(id = 0)]
	Aucroc,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BinaryClassificationMetrics {
	#[buffalo(id = 0, required)]
	pub auc_roc: f32,
	#[buffalo(id = 1, required)]
	pub default_threshold: BinaryClassificationMetricsForThreshold,
	#[buffalo(id = 2, required)]
	pub thresholds: Vec<BinaryClassificationMetricsForThreshold>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BinaryClassificationMetricsForThreshold {
	#[buffalo(id = 0, required)]
	pub threshold: f32,
	#[buffalo(id = 1, required)]
	pub true_positives: u64,
	#[buffalo(id = 2, required)]
	pub false_positives: u64,
	#[buffalo(id = 3, required)]
	pub true_negatives: u64,
	#[buffalo(id = 4, required)]
	pub false_negatives: u64,
	#[buffalo(id = 5, required)]
	pub accuracy: f32,
	#[buffalo(id = 6, required)]
	pub precision: Option<f32>,
	#[buffalo(id = 7, required)]
	pub recall: Option<f32>,
	#[buffalo(id = 8, required)]
	pub f1_score: Option<f32>,
	#[buffalo(id = 9, required)]
	pub true_positive_rate: f32,
	#[buffalo(id = 10, required)]
	pub false_positive_rate: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum BinaryClassificationModel {
	#[buffalo(id = 0)]
	Linear(LinearBinaryClassifier),
	#[buffalo(id = 1)]
	Tree(TreeBinaryClassifier),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LinearBinaryClassifier {
	#[buffalo(id = 0, required)]
	pub model: tangram_linear::serialize::BinaryClassifier,
	#[buffalo(id = 1, required)]
	pub train_options: LinearModelTrainOptions,
	#[buffalo(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[buffalo(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[buffalo(id = 4, required)]
	pub feature_importances: Vec<f32>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TreeBinaryClassifier {
	#[buffalo(id = 0, required)]
	pub model: tangram_tree::serialize::BinaryClassifier,
	#[buffalo(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[buffalo(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[buffalo(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[buffalo(id = 4, required)]
	pub feature_importances: Vec<f32>,
}
