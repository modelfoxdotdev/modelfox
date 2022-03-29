use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct MulticlassClassifier {
	#[buffalo(id = 0, required)]
	pub target_column_name: String,
	#[buffalo(id = 1, required)]
	pub classes: Vec<String>,
	#[buffalo(id = 2, required)]
	pub train_row_count: u64,
	#[buffalo(id = 3, required)]
	pub test_row_count: u64,
	#[buffalo(id = 4, required)]
	pub overall_row_count: u64,
	#[buffalo(id = 5, required)]
	pub stats_settings: StatsSettings,
	#[buffalo(id = 6, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 7, required)]
	pub overall_target_column_stats: ColumnStats,
	#[buffalo(id = 8, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 9, required)]
	pub train_target_column_stats: ColumnStats,
	#[buffalo(id = 10, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 11, required)]
	pub test_target_column_stats: ColumnStats,
	#[buffalo(id = 12, required)]
	pub baseline_metrics: MulticlassClassificationMetrics,
	#[buffalo(id = 13, required)]
	pub comparison_metric: MulticlassClassificationComparisonMetric,
	#[buffalo(id = 14, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[buffalo(id = 15, required)]
	pub best_grid_item_index: u64,
	#[buffalo(id = 16, required)]
	pub model: MulticlassClassificationModel,
	#[buffalo(id = 17, required)]
	pub test_metrics: MulticlassClassificationMetrics,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum MulticlassClassificationComparisonMetric {
	#[buffalo(id = 0)]
	Accuracy,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct MulticlassClassificationMetrics {
	#[buffalo(id = 0, required)]
	pub class_metrics: Vec<ClassMetrics>,
	#[buffalo(id = 1, required)]
	pub accuracy: f32,
	#[buffalo(id = 2, required)]
	pub precision_unweighted: f32,
	#[buffalo(id = 3, required)]
	pub precision_weighted: f32,
	#[buffalo(id = 4, required)]
	pub recall_unweighted: f32,
	#[buffalo(id = 5, required)]
	pub recall_weighted: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct ClassMetrics {
	#[buffalo(id = 0, required)]
	pub true_positives: u64,
	#[buffalo(id = 1, required)]
	pub false_positives: u64,
	#[buffalo(id = 2, required)]
	pub true_negatives: u64,
	#[buffalo(id = 3, required)]
	pub false_negatives: u64,
	#[buffalo(id = 4, required)]
	pub accuracy: f32,
	#[buffalo(id = 5, required)]
	pub precision: f32,
	#[buffalo(id = 6, required)]
	pub recall: f32,
	#[buffalo(id = 7, required)]
	pub f1_score: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum MulticlassClassificationModel {
	#[buffalo(id = 0)]
	Linear(LinearMulticlassClassifier),
	#[buffalo(id = 1)]
	Tree(TreeMulticlassClassifier),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LinearMulticlassClassifier {
	#[buffalo(id = 0, required)]
	pub model: modelfox_linear::serialize::MulticlassClassifier,
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
pub struct TreeMulticlassClassifier {
	#[buffalo(id = 0, required)]
	pub model: modelfox_tree::serialize::MulticlassClassifier,
	#[buffalo(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[buffalo(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[buffalo(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[buffalo(id = 4, required)]
	pub feature_importances: Vec<f32>,
}
