use crate::ModelTrainOptions;

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TrainGridItemOutput {
	#[buffalo(id = 0, required)]
	pub hyperparameters: ModelTrainOptions,
	#[buffalo(id = 1, required)]
	pub model_comparison_metric_value: f32,
	#[buffalo(id = 2, required)]
	pub duration: f32,
}
