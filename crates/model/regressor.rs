use crate::{
	ColumnStats, FeatureGroup, LinearModelTrainOptions, StatsSettings, TrainGridItemOutput,
	TreeModelTrainOptions,
};

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Regressor {
	#[buffalo(id = 0, required)]
	pub target_column_name: String,
	#[buffalo(id = 1, required)]
	pub train_row_count: u64,
	#[buffalo(id = 2, required)]
	pub test_row_count: u64,
	#[buffalo(id = 3, required)]
	pub overall_row_count: u64,
	#[buffalo(id = 4, required)]
	pub stats_settings: StatsSettings,
	#[buffalo(id = 5, required)]
	pub overall_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 6, required)]
	pub overall_target_column_stats: ColumnStats,
	#[buffalo(id = 7, required)]
	pub train_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 8, required)]
	pub train_target_column_stats: ColumnStats,
	#[buffalo(id = 9, required)]
	pub test_column_stats: Vec<ColumnStats>,
	#[buffalo(id = 10, required)]
	pub test_target_column_stats: ColumnStats,
	#[buffalo(id = 11, required)]
	pub baseline_metrics: RegressionMetrics,
	#[buffalo(id = 12, required)]
	pub comparison_metric: RegressionComparisonMetric,
	#[buffalo(id = 13, required)]
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	#[buffalo(id = 14, required)]
	pub best_grid_item_index: u64,
	#[buffalo(id = 15, required)]
	pub model: RegressionModel,
	#[buffalo(id = 16, required)]
	pub test_metrics: RegressionMetrics,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum RegressionModel {
	#[buffalo(id = 0)]
	Linear(LinearRegressor),
	#[buffalo(id = 1)]
	Tree(TreeRegressor),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct LinearRegressor {
	#[buffalo(id = 0, required)]
	pub model: tangram_linear::serialize::Regressor,
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
pub struct TreeRegressor {
	#[buffalo(id = 0, required)]
	pub model: tangram_tree::serialize::Regressor,
	#[buffalo(id = 1, required)]
	pub train_options: TreeModelTrainOptions,
	#[buffalo(id = 2, required)]
	pub feature_groups: Vec<FeatureGroup>,
	#[buffalo(id = 3, required)]
	pub losses: Option<Vec<f32>>,
	#[buffalo(id = 4, required)]
	pub feature_importances: Vec<f32>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum RegressionComparisonMetric {
	#[buffalo(id = 0)]
	MeanAbsoluteError,
	#[buffalo(id = 1)]
	MeanSquaredError,
	#[buffalo(id = 2)]
	RootMeanSquaredError,
	#[buffalo(id = 3)]
	R2,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct RegressionMetrics {
	#[buffalo(id = 0, required)]
	pub mse: f32,
	#[buffalo(id = 1, required)]
	pub rmse: f32,
	#[buffalo(id = 2, required)]
	pub mae: f32,
	#[buffalo(id = 3, required)]
	pub r2: f32,
}
