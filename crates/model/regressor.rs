use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct Regressor {
	#[tangram_serialize(id = 0, required)]
	pub target_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub train_row_count: u64,
	#[tangram_serialize(id = 2, required)]
	pub test_row_count: u64,
	#[tangram_serialize(id = 3, required)]
	pub overall_row_count: u64,
	#[tangram_serialize(id = 4, required)]
	pub stats_settings: StatsSettings,
	#[tangram_serialize(id = 5, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 6, required)]
	pub overall_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 7, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 8, required)]
	pub train_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 9, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[tangram_serialize(id = 10, required)]
	pub test_target_column_stats: ColumnStats,
	#[tangram_serialize(id = 11, required)]
	pub baseline_metrics: RegressionMetrics,
	#[tangram_serialize(id = 12, required)]
	pub comparison_metric: RegressionComparisonMetric,
	#[tangram_serialize(id = 13, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[tangram_serialize(id = 14, required)]
	pub best_grid_item_index: u64,
	#[tangram_serialize(id = 15, required)]
	pub model: RegressionModel,
	#[tangram_serialize(id = 16, required)]
	pub test_metrics: RegressionMetrics,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum RegressionModel {
	#[tangram_serialize(id = 0)]
	Linear(LinearRegressor),
	#[tangram_serialize(id = 1)]
	Tree(TreeRegressor),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct LinearRegressor {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_linear::serialize::Regressor,
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
pub struct TreeRegressor {
	#[tangram_serialize(id = 0, required)]
	pub model: tangram_tree::serialize::Regressor,
	#[tangram_serialize(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[tangram_serialize(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[tangram_serialize(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[tangram_serialize(id = 4, required)]
	pub feature_importances: Vec<f32>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum RegressionComparisonMetric {
	#[tangram_serialize(id = 0)]
	MeanAbsoluteError,
	#[tangram_serialize(id = 1)]
	MeanSquaredError,
	#[tangram_serialize(id = 2)]
	RootMeanSquaredError,
	#[tangram_serialize(id = 3)]
	R2,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct RegressionMetrics {
	#[tangram_serialize(id = 0, required)]
	pub mse: f32,
	#[tangram_serialize(id = 1, required)]
	pub rmse: f32,
	#[tangram_serialize(id = 2, required)]
	pub mae: f32,
	#[tangram_serialize(id = 3, required)]
	pub r2: f32,
}
