use crate::{
	config::{self, Config},
	grid,
	model::{
		BinaryClassificationComparisonMetric, BinaryClassificationModel, BinaryClassifier,
		ComparisonMetric, LinearBinaryClassificationModel, LinearMulticlassClassificationModel,
		LinearRegressionModel, Metrics, Model, ModelInner,
		MulticlassClassificationComparisonMetric, MulticlassClassificationModel,
		MulticlassClassifier, RegressionComparisonMetric, RegressionModel, Regressor, Task,
		TreeBinaryClassificationModel, TreeMulticlassClassificationModel, TreeRegressionModel,
	},
	progress::{
		LoadProgressEvent, ModelTestProgressEvent, ModelTrainProgressEvent, ProgressEvent,
		StatsProgressEvent, TrainGridItemProgressEvent, TrainProgressEvent,
	},
	stats::{ColumnStatsOutput, Stats, StatsSettings},
	test,
};
use ndarray::prelude::*;
use num::ToPrimitive;
use rand::{seq::SliceRandom, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use std::{
	collections::BTreeMap,
	path::Path,
	sync::Arc,
	time::{Duration, Instant},
	unreachable,
};
use tangram_error::{err, Result};
use tangram_id::Id;
use tangram_kill_chip::KillChip;
use tangram_progress_counter::ProgressCounter;
use tangram_table::prelude::*;

pub struct Trainer {
	id: Id,
	target_column_name: String,
	train_row_count: usize,
	test_row_count: usize,
	overall_row_count: usize,
	stats_settings: StatsSettings,
	overall_column_stats: Vec<ColumnStatsOutput>,
	overall_target_column_stats: ColumnStatsOutput,
	train_column_stats: Vec<ColumnStatsOutput>,
	train_target_column_stats: ColumnStatsOutput,
	test_column_stats: Vec<ColumnStatsOutput>,
	test_target_column_stats: ColumnStatsOutput,
	baseline_metrics: Metrics,
	comparison_metric: ComparisonMetric,
	dataset: Arc<Dataset>,
	grid: Vec<grid::GridItem>,
	task: Task,
}

impl Trainer {
	pub fn prepare(
		id: Id,
		file_path: Option<&Path>,
		file_path_train: Option<&Path>,
		file_path_test: Option<&Path>,
		target_column_name: &str,
		config_path: Option<&Path>,
		handle_progress_event: &mut dyn FnMut(ProgressEvent),
	) -> Result<Trainer> {
		// Load the config from the config file, if provided.
		let config: Option<Config> = load_config(config_path)?;

		// Load the train and test tables from the csv file(s).
		let dataset = match (file_path, file_path_train, file_path_test) {
			(Some(file_path), None, None) => Dataset::Train(load_and_shuffle_dataset_train(
				file_path,
				config.as_ref(),
				handle_progress_event,
			)?),
			(None, Some(file_path_train), Some(file_path_test)) => {
				Dataset::TrainAndTest(load_and_shuffle_dataset_train_and_test(
					file_path_train,
					file_path_test,
					config.as_ref(),
					handle_progress_event,
				)?)
			}
			_ => unreachable!(),
		};
		let (table_train, _, table_test) = dataset.split();

		// Retrieve the column names.
		let column_names: Vec<String> = table_train
			.columns()
			.iter()
			.map(|column| column.name().unwrap().to_owned())
			.collect();

		// Get the row counts.
		let train_row_count = table_train.nrows();
		let test_row_count = table_test.nrows();
		let overall_row_count = train_row_count + test_row_count;

		// Compute stats.
		let stats_settings = StatsSettings::default();
		let train_column_stats = Stats::compute(&table_train, &stats_settings, &mut |progress| {
			handle_progress_event(ProgressEvent::Stats(StatsProgressEvent::ComputeTrainStats(
				progress,
			)));
		});
		handle_progress_event(ProgressEvent::Stats(
			StatsProgressEvent::ComputeTrainStatsDone,
		));
		let test_column_stats = Stats::compute(&table_test, &stats_settings, &mut |progress| {
			handle_progress_event(ProgressEvent::Stats(StatsProgressEvent::ComputeTestStats(
				progress,
			)));
		});
		handle_progress_event(ProgressEvent::Stats(
			StatsProgressEvent::ComputeTestStatsDone,
		));
		handle_progress_event(ProgressEvent::Stats(StatsProgressEvent::Finalize));
		let overall_column_stats = train_column_stats.clone().merge(test_column_stats.clone());
		let mut train_column_stats = train_column_stats.finalize(&stats_settings).0;
		let mut test_column_stats = test_column_stats.finalize(&stats_settings).0;
		let mut overall_column_stats = overall_column_stats.finalize(&stats_settings).0;
		handle_progress_event(ProgressEvent::Stats(StatsProgressEvent::FinalizeDone));

		// Find the target column.
		let target_column_index = column_names
			.iter()
			.position(|column_name| *column_name == target_column_name)
			.ok_or_else(|| {
				err!(
					"did not find target column \"{}\" among column names \"{}\"",
					target_column_name,
					column_names.join(", ")
				)
			})?;

		// Pull out the target column from the column stats.
		let train_target_column_stats = train_column_stats.remove(target_column_index);
		let test_target_column_stats = test_column_stats.remove(target_column_index);
		let overall_target_column_stats = overall_column_stats.remove(target_column_index);

		// Determine the task.
		let task = match &overall_target_column_stats {
			ColumnStatsOutput::Number(_) => Task::Regression,
			ColumnStatsOutput::Enum(target_column) => match target_column.unique_count {
				2 => Task::BinaryClassification,
				_ => Task::MulticlassClassification,
			},
			_ => return Err(err!("invalid target column type")),
		};

		// Determine whether the target column contains invalid values.
		match overall_target_column_stats {
			ColumnStatsOutput::Number(stats) if stats.invalid_count != 0 => {
				return Err(err!("The target column contains invalid values."));
			}
			ColumnStatsOutput::Enum(stats) if stats.invalid_count != 0 => {
				return Err(err!("The target column contains invalid values."));
			}
			_ => {}
		};

		// Compute the baseline metrics.
		let progress_counter = ProgressCounter::new(train_row_count as u64);
		handle_progress_event(ProgressEvent::ComputeBaselineMetrics(
			progress_counter.clone(),
		));
		let baseline_metrics = compute_baseline_metrics(
			task,
			&table_train,
			target_column_index,
			&train_target_column_stats,
			&|| progress_counter.inc(1),
		);
		handle_progress_event(ProgressEvent::ComputeBaselineMetricsDone);

		// Choose the comparison metric.
		let comparison_metric = choose_comparison_metric(&config, &task)?;

		// Create the hyperparameter grid.
		let grid = compute_hyperparameter_grid(
			config.as_ref(),
			&task,
			target_column_index,
			&train_column_stats,
		);

		let trainer = Trainer {
			id,
			target_column_name: target_column_name.to_owned(),
			train_row_count,
			test_row_count,
			overall_row_count,
			stats_settings,
			overall_column_stats,
			overall_target_column_stats,
			train_column_stats,
			train_target_column_stats,
			test_column_stats,
			test_target_column_stats,
			baseline_metrics,
			comparison_metric,
			grid,
			task,
			dataset: Arc::new(dataset),
		};
		Ok(trainer)
	}

	/// Train each model in the grid and compute model comparison metrics.
	pub fn train_grid(
		&mut self,
		kill_chip: &KillChip,
		handle_progress_event: &mut dyn FnMut(ProgressEvent),
	) -> Result<Vec<TrainGridItemOutput>> {
		let (table_train, table_comparison, _) = self.dataset.split();
		let grid = &self.grid;
		let comparison_metric = self.comparison_metric;
		let train_grid_item_outputs = grid
			.iter()
			.cloned()
			.enumerate()
			.take_while(|_| !kill_chip.is_activated())
			.map(|(grid_item_index, grid_item)| {
				train_grid_item(
					grid.len(),
					grid_item_index,
					grid_item,
					&table_train,
					&table_comparison,
					comparison_metric,
					kill_chip,
					handle_progress_event,
				)
			})
			.collect();
		Ok(train_grid_item_outputs)
	}

	pub fn test_and_assemble_model(
		self,
		train_grid_item_outputs: Vec<TrainGridItemOutput>,
		handle_progress_event: &mut dyn FnMut(ProgressEvent),
	) -> Result<Model> {
		let Trainer {
			id,
			target_column_name,
			train_row_count,
			test_row_count,
			overall_row_count,
			stats_settings,
			overall_column_stats,
			overall_target_column_stats,
			train_column_stats,
			train_target_column_stats,
			test_column_stats,
			test_target_column_stats,
			baseline_metrics,
			comparison_metric,
			task,
			dataset,
			..
		} = self;

		let (_, _, table_test) = dataset.split();

		// Choose the best model.
		let (train_model_output, best_grid_item_index) =
			choose_best_model(&train_grid_item_outputs, &comparison_metric);

		// Test the best model.
		let test_metrics = test_model(&train_model_output, &table_test, &mut |_| {});

		handle_progress_event(ProgressEvent::Finalize);
		// Assemble the model.
		let inner = match task {
			Task::Regression => {
				let baseline_metrics = match baseline_metrics {
					Metrics::Regression(baseline_metrics) => baseline_metrics,
					_ => unreachable!(),
				};
				let comparison_metric = match comparison_metric {
					ComparisonMetric::Regression(comparison_metric) => comparison_metric,
					_ => unreachable!(),
				};
				let test_metrics = match test_metrics {
					Metrics::Regression(test_metrics) => test_metrics,
					_ => unreachable!(),
				};
				let model = match train_model_output {
					TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
						model,
						feature_groups,
						train_options,
						losses,
						feature_importances,
						..
					}) => RegressionModel::Linear(LinearRegressionModel {
						model,
						train_options,
						feature_groups,
						losses,
						feature_importances,
					}),
					TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
						model,
						feature_groups,
						train_options,
						losses,
						feature_importances,
						..
					}) => RegressionModel::Tree(TreeRegressionModel {
						model,
						train_options,
						feature_groups,
						losses,
						feature_importances,
					}),
					_ => unreachable!(),
				};
				ModelInner::Regressor(Regressor {
					target_column_name,
					train_row_count,
					test_row_count,
					overall_row_count,
					stats_settings,
					overall_column_stats,
					overall_target_column_stats,
					train_column_stats,
					train_target_column_stats,
					test_column_stats,
					test_target_column_stats,
					baseline_metrics,
					comparison_metric,
					train_grid_item_outputs,
					best_grid_item_index,
					model,
					test_metrics,
				})
			}
			Task::BinaryClassification => {
				let baseline_metrics = match baseline_metrics {
					Metrics::BinaryClassification(baseline_metrics) => baseline_metrics,
					_ => unreachable!(),
				};
				let comparison_metric = match comparison_metric {
					ComparisonMetric::BinaryClassification(comparison_metric) => comparison_metric,
					_ => unreachable!(),
				};
				let test_metrics = match test_metrics {
					Metrics::BinaryClassification(test_metrics) => test_metrics,
					_ => unreachable!(),
				};
				let model = match train_model_output {
					TrainModelOutput::LinearBinaryClassifier(
						LinearBinaryClassifierTrainModelOutput {
							model,
							feature_groups,
							losses,
							train_options,
							feature_importances,
							..
						},
					) => BinaryClassificationModel::Linear(LinearBinaryClassificationModel {
						model,
						train_options,
						feature_groups,
						losses,
						feature_importances,
					}),
					TrainModelOutput::TreeBinaryClassifier(
						TreeBinaryClassifierTrainModelOutput {
							model,
							feature_groups,
							losses,
							train_options,
							feature_importances,
							..
						},
					) => BinaryClassificationModel::Tree(TreeBinaryClassificationModel {
						model,
						train_options,
						feature_groups,
						losses,
						feature_importances,
					}),
					_ => unreachable!(),
				};
				let (negative_class, positive_class) = match &train_target_column_stats {
					ColumnStatsOutput::Enum(train_target_column_stats) => (
						train_target_column_stats.histogram[0].0.clone(),
						train_target_column_stats.histogram[1].0.clone(),
					),
					_ => unreachable!(),
				};
				ModelInner::BinaryClassifier(BinaryClassifier {
					target_column_name,
					negative_class,
					positive_class,
					train_row_count,
					test_row_count,
					overall_row_count,
					stats_settings,
					overall_column_stats,
					overall_target_column_stats,
					train_column_stats,
					train_target_column_stats,
					test_column_stats,
					test_target_column_stats,
					baseline_metrics,
					comparison_metric,
					train_grid_item_outputs,
					best_grid_item_index,
					model,
					test_metrics,
				})
			}
			Task::MulticlassClassification { .. } => {
				let baseline_metrics = match baseline_metrics {
					Metrics::MulticlassClassification(baseline_metrics) => baseline_metrics,
					_ => unreachable!(),
				};
				let comparison_metric = match comparison_metric {
					ComparisonMetric::MulticlassClassification(comparison_metric) => {
						comparison_metric
					}
					_ => unreachable!(),
				};
				let test_metrics = match test_metrics {
					Metrics::MulticlassClassification(test_metrics) => test_metrics,
					_ => unreachable!(),
				};
				let model = match train_model_output {
					TrainModelOutput::LinearMulticlassClassifier(
						LinearMulticlassClassifierTrainModelOutput {
							model,
							feature_groups,
							train_options,
							losses,
							feature_importances,
							..
						},
					) => {
						MulticlassClassificationModel::Linear(LinearMulticlassClassificationModel {
							model,
							train_options,
							feature_groups,
							losses,
							feature_importances,
						})
					}
					TrainModelOutput::TreeMulticlassClassifier(
						TreeMulticlassClassifierTrainModelOutput {
							model,
							feature_groups,
							train_options,
							losses,
							feature_importances,
							..
						},
					) => MulticlassClassificationModel::Tree(TreeMulticlassClassificationModel {
						model,
						train_options,
						feature_groups,
						losses,
						feature_importances,
					}),
					_ => unreachable!(),
				};
				let classes = match &train_target_column_stats {
					ColumnStatsOutput::Enum(train_target_column_stats) => train_target_column_stats
						.histogram
						.iter()
						.map(|(class, _)| class.clone())
						.collect(),
					_ => unreachable!(),
				};
				ModelInner::MulticlassClassifier(MulticlassClassifier {
					target_column_name,
					classes,
					train_row_count,
					test_row_count,
					overall_row_count,
					stats_settings,
					overall_column_stats,
					overall_target_column_stats,
					train_column_stats,
					train_target_column_stats,
					test_column_stats,
					test_target_column_stats,
					baseline_metrics,
					comparison_metric,
					train_grid_item_outputs,
					best_grid_item_index,
					model,
					test_metrics,
				})
			}
		};
		let model = Model {
			id,
			version: env!("CARGO_PKG_VERSION").to_owned(),
			date: chrono::Utc::now().to_rfc3339(),
			inner,
		};
		handle_progress_event(ProgressEvent::FinalizeDone);
		Ok(model)
	}
}

fn load_config(config_path: Option<&Path>) -> Result<Option<Config>> {
	if let Some(config_path) = config_path {
		let config = std::fs::read_to_string(config_path).map_err(|_| {
			err!(format!(
				"Could not find a config file at path: {:?}",
				config_path.to_str().unwrap()
			))
		})?;
		let config = serde_json::from_str(&config)?;
		Ok(Some(config))
	} else {
		Ok(None)
	}
}

enum Dataset {
	Train(DatasetTrain),
	TrainAndTest(DatasetTrainAndTest),
}

struct DatasetTrain {
	table: Table,
	comparison_fraction: f32,
	test_fraction: f32,
}

struct DatasetTrainAndTest {
	table_train: Table,
	table_test: Table,
	comparison_fraction: f32,
}

impl Dataset {
	fn split(&self) -> (TableView, TableView, TableView) {
		match self {
			Dataset::Train(DatasetTrain {
				table,
				comparison_fraction,
				test_fraction,
			}) => {
				let n_rows_test = (test_fraction * table.nrows().to_f32().unwrap())
					.floor()
					.to_usize()
					.unwrap();
				let n_rows_comparison = (comparison_fraction * table.nrows().to_f32().unwrap())
					.floor()
					.to_usize()
					.unwrap();
				let n_rows_train = table.nrows() - n_rows_test - n_rows_comparison;
				let (table_train, table_rest) = table.view().split_at_row(n_rows_train);
				let (table_comparison, table_test) = table_rest.split_at_row(n_rows_comparison);
				(table_train, table_comparison, table_test)
			}
			Dataset::TrainAndTest(DatasetTrainAndTest {
				table_train,
				table_test,
				comparison_fraction,
			}) => {
				let n_rows_comparison = (comparison_fraction
					* table_train.nrows().to_f32().unwrap())
				.floor()
				.to_usize()
				.unwrap();
				let n_rows_train = table_train.nrows() - n_rows_comparison;
				let (table_train, table_comparison) = table_train.view().split_at_row(n_rows_train);
				let table_test = table_test.view();
				(table_train, table_comparison, table_test)
			}
		}
	}
}

fn load_and_shuffle_dataset_train(
	file_path: &Path,
	config: Option<&Config>,
	handle_progress_event: &mut dyn FnMut(ProgressEvent),
) -> Result<DatasetTrain> {
	// Get the column types from the config, if set.
	let column_types = config.and_then(column_types_from_config);
	let mut table = Table::from_path(
		file_path,
		tangram_table::FromCsvOptions {
			column_types,
			infer_options: Default::default(),
			..Default::default()
		},
		&mut |progress_event| {
			handle_progress_event(ProgressEvent::Load(LoadProgressEvent::Train(
				progress_event,
			)))
		},
	)
	.map_err(|_| {
		err!(format!(
			"Could not find a train file at path: {:?}",
			file_path.to_str().unwrap()
		))
	})?;
	// Shuffle the table if enabled.
	shuffle_table(&mut table, config, handle_progress_event);
	// Split the table into train and test tables.
	let test_fraction = config
		.as_ref()
		.and_then(|config| config.test_fraction)
		.unwrap_or(config::DEFAULT_TEST_FRACTION);
	let comparison_fraction = config
		.as_ref()
		.and_then(|config| config.comparison_fraction)
		.unwrap_or(config::DEFAULT_COMPARISON_FRACTION);
	Ok(DatasetTrain {
		table,
		comparison_fraction,
		test_fraction,
	})
}

fn load_and_shuffle_dataset_train_and_test(
	file_path_train: &Path,
	file_path_test: &Path,
	config: Option<&Config>,
	handle_progress_event: &mut dyn FnMut(ProgressEvent),
) -> Result<DatasetTrainAndTest> {
	// Get the column types from the config, if set.
	let column_types = config.and_then(column_types_from_config);
	let mut table_train = Table::from_path(
		file_path_train,
		tangram_table::FromCsvOptions {
			column_types,
			infer_options: Default::default(),
			..Default::default()
		},
		&mut |progress_event| {
			handle_progress_event(ProgressEvent::Load(LoadProgressEvent::Train(
				progress_event,
			)))
		},
	)
	.map_err(|_| {
		err!(format!(
			"Could not find a train file at path: {:?}",
			file_path_train.to_str().unwrap()
		))
	})?;
	// Force the column types for table_test to be the same as table_train.
	let column_types = table_train
		.columns()
		.iter()
		.map(|column| match column {
			TableColumn::Unknown(column) => {
				(column.name().to_owned().unwrap(), TableColumnType::Unknown)
			}
			TableColumn::Enum(column) => (
				column.name().to_owned().unwrap(),
				TableColumnType::Enum {
					variants: column.variants().to_owned(),
				},
			),
			TableColumn::Number(column) => {
				(column.name().to_owned().unwrap(), TableColumnType::Number)
			}
			TableColumn::Text(column) => (column.name().to_owned().unwrap(), TableColumnType::Text),
		})
		.collect();
	let table_test = Table::from_path(
		file_path_test,
		tangram_table::FromCsvOptions {
			column_types: Some(column_types),
			infer_options: Default::default(),
			..Default::default()
		},
		&mut |progress_event| {
			handle_progress_event(ProgressEvent::Load(LoadProgressEvent::Test(progress_event)))
		},
	)
	.map_err(|_| {
		err!(format!(
			"Could not find a test file at path: {:?}",
			file_path_test.to_str().unwrap()
		))
	})?;
	let comparison_fraction = config
		.as_ref()
		.and_then(|config| config.comparison_fraction)
		.unwrap_or(config::DEFAULT_COMPARISON_FRACTION);
	shuffle_table(&mut table_train, config, handle_progress_event);
	Ok(DatasetTrainAndTest {
		table_train,
		table_test,
		comparison_fraction,
	})
}

fn column_types_from_config(config: &Config) -> Option<BTreeMap<String, TableColumnType>> {
	config.column_types.as_ref().map(|column_types| {
		column_types
			.iter()
			.map(|(column_name, column_type)| {
				let column_type = match column_type {
					config::ColumnType::Unknown => TableColumnType::Unknown,
					config::ColumnType::Number => TableColumnType::Number,
					config::ColumnType::Enum(column_type) => TableColumnType::Enum {
						variants: column_type.variants.clone(),
					},
					config::ColumnType::Text => TableColumnType::Text,
				};
				(column_name.clone(), column_type)
			})
			.collect()
	})
}

const DEFAULT_SEED: u64 = 42;

fn shuffle_table(
	table: &mut Table,
	config: Option<&Config>,
	handle_progress_event: &mut dyn FnMut(ProgressEvent),
) {
	// Check if shuffling is enabled in the config. If it is, use the seed from the config.
	let seed = config
		.as_ref()
		.and_then(|config| config.shuffle.as_ref())
		.map(|shuffle| match shuffle {
			config::Shuffle::Enabled(enabled) => {
				if *enabled {
					Some(DEFAULT_SEED)
				} else {
					None
				}
			}
			config::Shuffle::Options { seed } => Some(*seed),
		})
		.unwrap_or(Some(DEFAULT_SEED));
	// Shuffle the table.
	if let Some(seed) = seed {
		handle_progress_event(ProgressEvent::Load(LoadProgressEvent::Shuffle));
		for column in table.columns_mut().iter_mut() {
			let mut rng = Xoshiro256Plus::seed_from_u64(seed);
			match column {
				TableColumn::Unknown(_) => {}
				TableColumn::Number(column) => column.data_mut().shuffle(&mut rng),
				TableColumn::Enum(column) => column.data_mut().shuffle(&mut rng),
				TableColumn::Text(column) => column.data_mut().shuffle(&mut rng),
			}
		}
		handle_progress_event(ProgressEvent::Load(LoadProgressEvent::ShuffleDone));
	}
}

fn compute_hyperparameter_grid(
	config: Option<&Config>,
	task: &Task,
	target_column_index: usize,
	train_column_stats: &[ColumnStatsOutput],
) -> Vec<grid::GridItem> {
	config
		.as_ref()
		.and_then(|config| config.grid.as_ref())
		.map(|grid| match &task {
			Task::Regression => grid::compute_regression_hyperparameter_grid(
				grid,
				target_column_index,
				&train_column_stats,
			),
			Task::BinaryClassification => grid::compute_binary_classification_hyperparameter_grid(
				grid,
				target_column_index,
				&train_column_stats,
			),
			Task::MulticlassClassification { .. } => {
				grid::compute_multiclass_classification_hyperparameter_grid(
					grid,
					target_column_index,
					&train_column_stats,
				)
			}
		})
		.unwrap_or_else(|| match &task {
			Task::Regression => grid::default_regression_hyperparameter_grid(
				target_column_index,
				&train_column_stats,
			),
			Task::BinaryClassification => grid::default_binary_classification_hyperparameter_grid(
				target_column_index,
				&train_column_stats,
			),
			Task::MulticlassClassification { .. } => {
				grid::default_multiclass_classification_hyperparameter_grid(
					target_column_index,
					&train_column_stats,
				)
			}
		})
}

fn compute_baseline_metrics(
	task: Task,
	table_train: &TableView,
	target_column_index: usize,
	train_target_column_stats: &ColumnStatsOutput,
	progress: &impl Fn(),
) -> Metrics {
	match task {
		Task::Regression => {
			let labels = table_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_number().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				ColumnStatsOutput::Number(train_target_column_stats) => train_target_column_stats,
				_ => unreachable!(),
			};
			let baseline_prediction = train_target_column_stats.mean;
			let mut metrics = tangram_metrics::RegressionMetrics::new();
			for label in labels.iter() {
				metrics.update(tangram_metrics::RegressionMetricsInput {
					predictions: &[baseline_prediction],
					labels: &[*label],
				});
				progress();
			}
			Metrics::Regression(metrics.finalize())
		}
		Task::BinaryClassification => {
			let labels = table_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_enum().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				ColumnStatsOutput::Enum(train_target_column_stats) => train_target_column_stats,
				_ => unreachable!(),
			};
			let total_count = train_target_column_stats.count.to_f32().unwrap();
			let baseline_probability = train_target_column_stats
				.histogram
				.iter()
				.last()
				.unwrap()
				.1
				.to_f32()
				.unwrap() / total_count;
			let mut metrics = tangram_metrics::BinaryClassificationMetrics::new(3);
			for label in labels.iter() {
				metrics.update(tangram_metrics::BinaryClassificationMetricsInput {
					probabilities: &[baseline_probability],
					labels: &[*label],
				});
				progress();
			}
			Metrics::BinaryClassification(metrics.finalize())
		}
		Task::MulticlassClassification => {
			let labels = table_train.columns().get(target_column_index).unwrap();
			let labels = labels.as_enum().unwrap();
			let train_target_column_stats = match &train_target_column_stats {
				ColumnStatsOutput::Enum(train_target_column_stats) => train_target_column_stats,
				_ => unreachable!(),
			};
			let total_count = train_target_column_stats.count.to_f32().unwrap();
			let baseline_probabilities = train_target_column_stats
				.histogram
				.iter()
				.map(|(_, count)| count.to_f32().unwrap() / total_count)
				.collect::<Vec<_>>();
			let mut metrics = tangram_metrics::MulticlassClassificationMetrics::new(
				train_target_column_stats.histogram.len(),
			);
			for label in labels.iter() {
				metrics.update(tangram_metrics::MulticlassClassificationMetricsInput {
					probabilities: ArrayView::from(baseline_probabilities.as_slice())
						.insert_axis(Axis(0)),
					labels: ArrayView::from(&[*label]),
				});
				progress();
			}
			Metrics::MulticlassClassification(metrics.finalize())
		}
	}
}

pub struct TrainGridItemOutput {
	pub train_model_output: TrainModelOutput,
	pub model_comparison_metrics: Metrics,
	pub model_comparison_metric_value: f32,
	pub duration: Duration,
}

#[allow(clippy::clippy::too_many_arguments)]
fn train_grid_item(
	grid_item_count: usize,
	grid_item_index: usize,
	grid_item: grid::GridItem,
	table_train: &TableView,
	table_comparison: &TableView,
	comparison_metric: ComparisonMetric,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(ProgressEvent),
) -> TrainGridItemOutput {
	let start = Instant::now();
	let train_model_output = train_model(grid_item, table_train, kill_chip, &mut |progress| {
		handle_progress_event(ProgressEvent::Train(TrainProgressEvent {
			grid_item_index,
			grid_item_count,
			grid_item_progress_event: progress,
		}))
	});
	let duration = start.elapsed();
	let model_comparison_metrics =
		compute_model_comparison_metrics(&train_model_output, &table_comparison, &mut |progress| {
			handle_progress_event(ProgressEvent::Train(TrainProgressEvent {
				grid_item_index,
				grid_item_count,
				grid_item_progress_event: TrainGridItemProgressEvent::ComputeModelComparisonMetrics(
					progress,
				),
			}))
		});
	let model_comparison_metric_value =
		get_model_comparison_metric_value(&model_comparison_metrics, comparison_metric);
	TrainGridItemOutput {
		train_model_output,
		model_comparison_metrics,
		model_comparison_metric_value,
		duration,
	}
}

fn get_model_comparison_metric_value(
	metrics: &Metrics,
	comparison_metric: ComparisonMetric,
) -> f32 {
	match (comparison_metric, metrics) {
		(ComparisonMetric::Regression(comparison_metric), Metrics::Regression(metrics)) => {
			match comparison_metric {
				RegressionComparisonMetric::MeanAbsoluteError => metrics.mae,
				RegressionComparisonMetric::MeanSquaredError => metrics.mse,
				RegressionComparisonMetric::RootMeanSquaredError => metrics.rmse,
				RegressionComparisonMetric::R2 => metrics.r2,
			}
		}
		(
			ComparisonMetric::BinaryClassification(comparison_metric),
			Metrics::BinaryClassification(metrics),
		) => match comparison_metric {
			BinaryClassificationComparisonMetric::AucRoc => metrics.auc_roc_approx,
		},
		(
			ComparisonMetric::MulticlassClassification(comparison_metric),
			Metrics::MulticlassClassification(metrics),
		) => match comparison_metric {
			MulticlassClassificationComparisonMetric::Accuracy => metrics.accuracy,
		},
		_ => unreachable!(),
	}
}

#[derive(Clone, Debug)]
pub enum TrainModelOutput {
	LinearRegressor(LinearRegressorTrainModelOutput),
	TreeRegressor(TreeRegressorTrainModelOutput),
	LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput),
	TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput),
	LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput),
	TreeMulticlassClassifier(TreeMulticlassClassifierTrainModelOutput),
}

#[derive(Clone, Debug)]
pub struct LinearRegressorTrainModelOutput {
	pub model: tangram_linear::Regressor,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct TreeRegressorTrainModelOutput {
	pub model: tangram_tree::Regressor,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct LinearBinaryClassifierTrainModelOutput {
	pub model: tangram_linear::BinaryClassifier,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct TreeBinaryClassifierTrainModelOutput {
	pub model: tangram_tree::BinaryClassifier,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct LinearMulticlassClassifierTrainModelOutput {
	pub model: tangram_linear::MulticlassClassifier,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct TreeMulticlassClassifierTrainModelOutput {
	pub model: tangram_tree::MulticlassClassifier,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub target_column_index: usize,
	pub losses: Option<Vec<f32>>,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_importances: Vec<f32>,
}

fn train_model(
	grid_item: grid::GridItem,
	table_train: &TableView,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	match grid_item {
		grid::GridItem::LinearRegressor {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_regressor(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
		grid::GridItem::TreeRegressor {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_regressor(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
		grid::GridItem::LinearBinaryClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_binary_classifier(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
		grid::GridItem::TreeBinaryClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_binary_classifier(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
		grid::GridItem::LinearMulticlassClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_linear_multiclass_classifier(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
		grid::GridItem::TreeMulticlassClassifier {
			target_column_index,
			feature_groups,
			options,
		} => train_tree_multiclass_classifier(
			table_train,
			target_column_index,
			feature_groups,
			options,
			kill_chip,
			handle_progress_event,
		),
	}
}

fn train_linear_regressor(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_rows = table_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Linear(progress),
		))
	};
	let progress = tangram_linear::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output =
		tangram_linear::Regressor::train(features.view(), labels, &linear_options, progress);
	TrainModelOutput::LinearRegressor(LinearRegressorTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_regressor(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features as u64;
	let n_rows = table_train.nrows() as u64;
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features = tangram_features::compute_features_table(table_train, &feature_groups, &|i| {
		progress_counter.inc(i)
	});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_number()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Tree(progress),
		))
	};
	let progress = tangram_tree::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output =
		tangram_tree::Regressor::train(features.view(), labels, &tree_options, progress);
	TrainModelOutput::TreeRegressor(TreeRegressorTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_linear_binary_classifier(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_rows = table_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Linear(progress),
		))
	};
	let progress = tangram_linear::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output =
		tangram_linear::BinaryClassifier::train(features.view(), labels, &linear_options, progress);
	TrainModelOutput::LinearBinaryClassifier(LinearBinaryClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_binary_classifier(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_rows = table_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features = tangram_features::compute_features_table(table_train, &feature_groups, &|i| {
		progress_counter.inc(i)
	});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Tree(progress),
		))
	};
	let progress = tangram_tree::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output =
		tangram_tree::BinaryClassifier::train(features.view(), labels, &tree_options, progress);
	TrainModelOutput::TreeBinaryClassifier(TreeBinaryClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_linear_multiclass_classifier(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::LinearModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_rows = table_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_train, &feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let linear_options = compute_linear_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Linear(progress),
		))
	};
	let progress = tangram_linear::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output = tangram_linear::MulticlassClassifier::train(
		features.view(),
		labels,
		&linear_options,
		progress,
	);
	TrainModelOutput::LinearMulticlassClassifier(LinearMulticlassClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: linear_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn train_tree_multiclass_classifier(
	table_train: &TableView,
	target_column_index: usize,
	feature_groups: Vec<tangram_features::FeatureGroup>,
	options: grid::TreeModelTrainOptions,
	kill_chip: &KillChip,
	handle_progress_event: &mut dyn FnMut(TrainGridItemProgressEvent),
) -> TrainModelOutput {
	let n_features = feature_groups.iter().map(|f| f.n_features()).sum::<usize>();
	let n_features = n_features.to_u64().unwrap();
	let n_rows = table_train.nrows().to_u64().unwrap();
	let progress_counter = ProgressCounter::new(n_features * n_rows);
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features = tangram_features::compute_features_table(table_train, &feature_groups, &|i| {
		progress_counter.inc(i)
	});
	handle_progress_event(TrainGridItemProgressEvent::ComputeFeaturesDone);
	let labels = table_train
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap()
		.clone();
	let tree_options = compute_tree_options(&options);
	let progress = &mut |progress| {
		handle_progress_event(TrainGridItemProgressEvent::TrainModel(
			ModelTrainProgressEvent::Tree(progress),
		))
	};
	let progress = tangram_tree::Progress {
		kill_chip,
		handle_progress_event: progress,
	};
	let train_output =
		tangram_tree::MulticlassClassifier::train(features.view(), labels, &tree_options, progress);
	TrainModelOutput::TreeMulticlassClassifier(TreeMulticlassClassifierTrainModelOutput {
		model: train_output.model,
		feature_groups,
		target_column_index,
		train_options: tree_options,
		losses: train_output.losses,
		feature_importances: train_output.feature_importances.unwrap(),
	})
}

fn compute_linear_options(options: &grid::LinearModelTrainOptions) -> tangram_linear::TrainOptions {
	let mut linear_options = tangram_linear::TrainOptions {
		compute_losses: true,
		..Default::default()
	};
	if let Some(l2_regularization) = options.l2_regularization {
		linear_options.l2_regularization = l2_regularization;
	}
	if let Some(learning_rate) = options.learning_rate {
		linear_options.learning_rate = learning_rate;
	}
	if let Some(max_epochs) = options.max_epochs {
		linear_options.max_epochs = max_epochs.to_usize().unwrap();
	}
	if let Some(n_examples_per_batch) = options.n_examples_per_batch {
		linear_options.n_examples_per_batch = n_examples_per_batch.to_usize().unwrap();
	}
	if let Some(early_stopping_options) = options.early_stopping_options.as_ref() {
		linear_options.early_stopping_options = Some(tangram_linear::EarlyStoppingOptions {
			early_stopping_fraction: early_stopping_options.early_stopping_fraction,
			min_decrease_in_loss_for_significant_change: early_stopping_options
				.early_stopping_threshold,
			n_rounds_without_improvement_to_stop: early_stopping_options.early_stopping_rounds,
		})
	}
	linear_options
}

fn compute_tree_options(options: &grid::TreeModelTrainOptions) -> tangram_tree::TrainOptions {
	let mut tree_options = tangram_tree::TrainOptions {
		compute_losses: true,
		..Default::default()
	};
	if let Some(early_stopping_options) = options.early_stopping_options.as_ref() {
		tree_options.early_stopping_options = Some(tangram_tree::EarlyStoppingOptions {
			early_stopping_fraction: early_stopping_options.early_stopping_fraction,
			n_rounds_without_improvement_to_stop: early_stopping_options.early_stopping_rounds,
			min_decrease_in_loss_for_significant_change: early_stopping_options
				.early_stopping_threshold,
		})
	}
	if let Some(l2_regularization_for_continuous_splits) =
		options.l2_regularization_for_continuous_splits
	{
		tree_options.l2_regularization_for_continuous_splits =
			l2_regularization_for_continuous_splits;
	}
	if let Some(l2_regularization_for_discrete_splits) =
		options.l2_regularization_for_discrete_splits
	{
		tree_options.l2_regularization_for_discrete_splits = l2_regularization_for_discrete_splits;
	}
	if let Some(learning_rate) = options.learning_rate {
		tree_options.learning_rate = learning_rate;
	}
	if let Some(max_depth) = options.max_depth {
		tree_options.max_depth = Some(max_depth.to_usize().unwrap());
	}
	if let Some(max_examples_for_computing_bin_thresholds) =
		options.max_examples_for_computing_bin_thresholds
	{
		tree_options.max_examples_for_computing_bin_thresholds =
			max_examples_for_computing_bin_thresholds
				.to_usize()
				.unwrap();
	}
	if let Some(max_leaf_nodes) = options.max_leaf_nodes {
		tree_options.max_leaf_nodes = max_leaf_nodes.to_usize().unwrap();
	}
	if let Some(max_rounds) = options.max_rounds {
		tree_options.max_rounds = max_rounds.to_usize().unwrap();
	}
	if let Some(max_valid_bins_for_number_features) = options.max_valid_bins_for_number_features {
		tree_options.max_valid_bins_for_number_features = max_valid_bins_for_number_features;
	}
	if let Some(min_examples_per_node) = options.min_examples_per_node {
		tree_options.min_examples_per_node = min_examples_per_node.to_usize().unwrap();
	}
	if let Some(min_gain_to_split) = options.min_gain_to_split {
		tree_options.min_gain_to_split = min_gain_to_split;
	}
	if let Some(min_sum_hessians_per_node) = options.min_sum_hessians_per_node {
		tree_options.min_sum_hessians_per_node = min_sum_hessians_per_node;
	}
	if let Some(smoothing_factor_for_discrete_bin_sorting) =
		options.smoothing_factor_for_discrete_bin_sorting
	{
		tree_options.smoothing_factor_for_discrete_bin_sorting =
			smoothing_factor_for_discrete_bin_sorting;
	}
	tree_options
}

fn choose_comparison_metric(config: &Option<Config>, task: &Task) -> Result<ComparisonMetric> {
	match task {
		Task::Regression => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::Mae => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::MeanAbsoluteError,
					)),
					config::ComparisonMetric::Mse => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::MeanSquaredError,
					)),
					config::ComparisonMetric::Rmse => Ok(ComparisonMetric::Regression(
						RegressionComparisonMetric::RootMeanSquaredError,
					)),
					config::ComparisonMetric::R2 => {
						Ok(ComparisonMetric::Regression(RegressionComparisonMetric::R2))
					}
					metric => Err(err!(
						"{} is an invalid model comparison metric for regression",
						metric
					)),
				}
			} else {
				Ok(ComparisonMetric::Regression(
					RegressionComparisonMetric::RootMeanSquaredError,
				))
			}
		}
		Task::BinaryClassification => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::Accuracy => {
						Ok(ComparisonMetric::BinaryClassification(
							BinaryClassificationComparisonMetric::AucRoc,
						))
					}
					metric => Err(err!(
						"{} is an invalid model comparison metric for binary classification",
						metric,
					)),
				}
			} else {
				Ok(ComparisonMetric::BinaryClassification(
					BinaryClassificationComparisonMetric::AucRoc,
				))
			}
		}
		Task::MulticlassClassification { .. } => {
			if let Some(metric) = config
				.as_ref()
				.and_then(|config| config.comparison_metric.as_ref())
			{
				match metric {
					config::ComparisonMetric::Accuracy => {
						Ok(ComparisonMetric::MulticlassClassification(
							MulticlassClassificationComparisonMetric::Accuracy,
						))
					}
					metric => Err(err!(
						"{} is an invalid model comparison metric for multiclass classification",
						metric,
					)),
				}
			} else {
				Ok(ComparisonMetric::MulticlassClassification(
					MulticlassClassificationComparisonMetric::Accuracy,
				))
			}
		}
	}
}

fn compute_model_comparison_metrics(
	train_model_output: &TrainModelOutput,
	table_comparison: &TableView,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> Metrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_linear_regressor(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::Regression(metrics)
		}
		TrainModelOutput::TreeRegressor(train_model_output) => {
			let TreeRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_tree_regressor(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::Regression(metrics)
		}
		TrainModelOutput::LinearBinaryClassifier(train_model_output) => {
			let LinearBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_linear_binary_classifier(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::BinaryClassification(metrics)
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_tree_binary_classifier(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::BinaryClassification(metrics)
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_linear_multiclass_classifier(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::MulticlassClassification(metrics)
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let metrics = test::test_tree_multiclass_classifier(
				&table_comparison,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::MulticlassClassification(metrics)
		}
	}
}

fn choose_best_model(
	outputs: &[TrainGridItemOutput],
	comparison_metric: &ComparisonMetric,
) -> (TrainModelOutput, usize) {
	match comparison_metric {
		ComparisonMetric::Regression(comparison_metric) => {
			choose_best_model_regression(outputs, comparison_metric)
		}
		ComparisonMetric::BinaryClassification(comparison_metric) => {
			choose_best_model_binary_classification(outputs, comparison_metric)
		}
		ComparisonMetric::MulticlassClassification(comparison_metric) => {
			choose_best_model_multiclass_classification(outputs, comparison_metric)
		}
	}
}

fn choose_best_model_regression(
	outputs: &[TrainGridItemOutput],
	comparison_metric: &RegressionComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.iter()
		.enumerate()
		.max_by(|(_, output_a), (_, output_b)| {
			let metrics_a = match &output_a.model_comparison_metrics {
				Metrics::Regression(metrics) => metrics,
				_ => unreachable!(),
			};
			let metrics_b = match &output_b.model_comparison_metrics {
				Metrics::Regression(metrics) => metrics,
				_ => unreachable!(),
			};
			match comparison_metric {
				RegressionComparisonMetric::MeanAbsoluteError => {
					metrics_b.mae.partial_cmp(&metrics_a.mae).unwrap()
				}
				RegressionComparisonMetric::RootMeanSquaredError => {
					metrics_b.rmse.partial_cmp(&metrics_a.rmse).unwrap()
				}
				RegressionComparisonMetric::MeanSquaredError => {
					metrics_b.mse.partial_cmp(&metrics_a.mse).unwrap()
				}
				RegressionComparisonMetric::R2 => metrics_a.r2.partial_cmp(&metrics_b.r2).unwrap(),
			}
		})
		.map(|(index, output)| (output.train_model_output.clone(), index))
		.unwrap()
}

fn choose_best_model_binary_classification(
	outputs: &[TrainGridItemOutput],
	comparison_metric: &BinaryClassificationComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.iter()
		.enumerate()
		.max_by(|(_, output_a), (_, output_b)| {
			let metrics_a = match &output_a.model_comparison_metrics {
				Metrics::BinaryClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			let metrics_b = match &output_b.model_comparison_metrics {
				Metrics::BinaryClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			match comparison_metric {
				BinaryClassificationComparisonMetric::AucRoc => metrics_a
					.auc_roc_approx
					.partial_cmp(&metrics_b.auc_roc_approx)
					.unwrap(),
			}
		})
		.map(|(index, output)| (output.train_model_output.clone(), index))
		.unwrap()
}

fn choose_best_model_multiclass_classification(
	outputs: &[TrainGridItemOutput],
	comparison_metric: &MulticlassClassificationComparisonMetric,
) -> (TrainModelOutput, usize) {
	outputs
		.iter()
		.enumerate()
		.max_by(|(_, output_a), (_, output_b)| {
			let metrics_a = match &output_a.model_comparison_metrics {
				Metrics::MulticlassClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			let metrics_b = match &output_b.model_comparison_metrics {
				Metrics::MulticlassClassification(metrics) => metrics,
				_ => unreachable!(),
			};
			match comparison_metric {
				MulticlassClassificationComparisonMetric::Accuracy => {
					metrics_a.accuracy.partial_cmp(&metrics_b.accuracy).unwrap()
				}
			}
		})
		.map(|(index, output)| (output.train_model_output.clone(), index))
		.unwrap()
}

fn test_model(
	train_model_output: &TrainModelOutput,
	table_test: &TableView,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> Metrics {
	match train_model_output {
		TrainModelOutput::LinearRegressor(train_model_output) => {
			let LinearRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_regressor(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::Regression(test_metrics)
		}
		TrainModelOutput::TreeRegressor(train_model_output) => {
			let TreeRegressorTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_regressor(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::Regression(test_metrics)
		}
		TrainModelOutput::LinearBinaryClassifier(train_model_output) => {
			let LinearBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_binary_classifier(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::BinaryClassification(test_metrics)
		}
		TrainModelOutput::TreeBinaryClassifier(train_model_output) => {
			let TreeBinaryClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_binary_classifier(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::BinaryClassification(test_metrics)
		}
		TrainModelOutput::LinearMulticlassClassifier(train_model_output) => {
			let LinearMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_linear_multiclass_classifier(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::MulticlassClassification(test_metrics)
		}
		TrainModelOutput::TreeMulticlassClassifier(train_model_output) => {
			let TreeMulticlassClassifierTrainModelOutput {
				target_column_index,
				feature_groups,
				model,
				..
			} = &train_model_output;
			let test_metrics = test::test_tree_multiclass_classifier(
				&table_test,
				*target_column_index,
				feature_groups,
				model,
				handle_progress_event,
			);
			Metrics::MulticlassClassification(test_metrics)
		}
	}
}
