use crate::ModelTrainOptions;

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TrainGridItemOutput {
	#[tangram_serialize(id = 0, required)]
	pub hyperparameters: ModelTrainOptions,
	#[tangram_serialize(id = 1, required)]
	pub model_comparison_metric_value: f32,
	#[tangram_serialize(id = 2, required)]
	pub duration: f32,
}
