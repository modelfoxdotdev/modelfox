pub use modelfox_linear::TrainProgressEvent as LinearTrainProgressEvent;
pub use modelfox_tree::TrainProgressEvent as TreeTrainProgressEvent;
use modelfox_progress_counter::ProgressCounter;

#[derive(Clone, Debug)]
pub enum ProgressEvent {
	Info(String),
	Warning(String),
	Load(LoadProgressEvent),
	Stats(StatsProgressEvent),
	ComputeBaselineMetrics(ProgressCounter),
	ComputeBaselineMetricsDone,
	Train(TrainProgressEvent),
	Test(ModelTestProgressEvent),
	Finalize,
	FinalizeDone,
}

#[derive(Clone, Debug)]
pub enum LoadProgressEvent {
	Train(modelfox_table::LoadProgressEvent),
	Test(modelfox_table::LoadProgressEvent),
	Shuffle,
	ShuffleDone,
}

#[derive(Clone, Debug)]
pub enum StatsProgressEvent {
	ComputeTrainStats(ProgressCounter),
	ComputeTrainStatsDone,
	ComputeTestStats(ProgressCounter),
	ComputeTestStatsDone,
	Finalize,
	FinalizeDone,
}

#[derive(Clone, Debug)]
pub struct TrainProgressEvent {
	pub grid_item_index: usize,
	pub grid_item_count: usize,
	pub grid_item_progress_event: TrainGridItemProgressEvent,
}

#[derive(Clone, Debug)]
pub enum TrainGridItemProgressEvent {
	ComputeFeatures(ProgressCounter),
	ComputeFeaturesDone,
	TrainModel(ModelTrainProgressEvent),
	ComputeModelComparisonMetrics(ModelTestProgressEvent),
}

#[derive(Clone, Debug)]
pub enum ModelTrainProgressEvent {
	Linear(modelfox_linear::TrainProgressEvent),
	Tree(modelfox_tree::TrainProgressEvent),
}

#[derive(Clone, Debug)]
pub enum ModelTestProgressEvent {
	ComputeFeatures(ProgressCounter),
	ComputeFeaturesDone,
	Test(ProgressCounter),
	TestDone,
}
