use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct MulticlassClassifier {
	#[tangram_serialize(id = 0, required)]
	pub target_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub classes: Vec<String>,
	#[tangram_serialize(id = 2, required)]
	pub train_row_count: u64,
	#[tangram_serialize(id = 3, required)]
	pub test_row_count: u64,
	#[tangram_serialize(id = 4, required)]
	pub overall_row_count: u64,
	#[tangram_serialize(id = 5, required)]
	pub stats_settings: StatsSettings,
	#[tangram_serialize(id = 6, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 7, required)]
	pub overall_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 8, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 9, required)]
	pub train_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 10, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 11, required)]
	pub test_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 12, required)]
	pub baseline_metrics: MulticlassClassificationMetrics,
	#[tangram_serialize(id = 13, required)]
	pub comparison_metric: MulticlassClassificationComparisonMetric,
	#[tangram_serialize(id = 14, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[tangram_serialize(id = 15, required)]
	pub best_grid_item_index: u64,
	#[tangram_serialize(id = 16, required)]
	pub model: MulticlassClassificationModel,
	#[tangram_serialize(id = 17, required)]
	pub test_metrics: MulticlassClassificationMetrics,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum MulticlassClassificationComparisonMetric {
	#[tangram_serialize(id = 0)]
	Accuracy,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct MulticlassClassificationMetrics {
	#[tangram_serialize(id = 0, required)]
	pub class_metrics: Vec<ClassMetrics>,
	#[tangram_serialize(id = 1, required)]
	pub accuracy: f32,
	#[tangram_serialize(id = 2, required)]
	pub precision_unweighted: f32,
	#[tangram_serialize(id = 3, required)]
	pub precision_weighted: f32,
	#[tangram_serialize(id = 4, required)]
	pub recall_unweighted: f32,
	#[tangram_serialize(id = 5, required)]
	pub recall_weighted: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct ClassMetrics {
	#[tangram_serialize(id = 0, required)]
	pub true_positives: u64,
	#[tangram_serialize(id = 1, required)]
	pub false_positives: u64,
	#[tangram_serialize(id = 2, required)]
	pub true_negatives: u64,
	#[tangram_serialize(id = 3, required)]
	pub false_negatives: u64,
	#[tangram_serialize(id = 4, required)]
	pub accuracy: f32,
	#[tangram_serialize(id = 5, required)]
	pub precision: f32,
	#[tangram_serialize(id = 6, required)]
	pub recall: f32,
	#[tangram_serialize(id = 7, required)]
	pub f1_score: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum MulticlassClassificationModel {
	#[tangram_serialize(id = 0)]
	Linear(LinearMulticlassClassifier),
	#[tangram_serialize(id = 1)]
	Tree(TreeMulticlassClassifier),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct LinearMulticlassClassifier {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_linear::serialize::MulticlassClassifier,
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
pub struct TreeMulticlassClassifier {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_tree::serialize::MulticlassClassifier,
	#[tangram_serialize(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[tangram_serialize(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[tangram_serialize(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[tangram_serialize(id = 4, required)]
	pub feature_importances: Vec<f32>,
}
