use crate::progress::ModelTestProgressEvent;
use ndarray::prelude::*;
use tangram_progress_counter::ProgressCounter;
use tangram_table::prelude::*;
use tangram_zip::zip;

pub fn test_linear_regressor(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::Regressor,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let labels = table_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	struct State {
		predictions: Array1<f32>,
		test_metrics: tangram_metrics::RegressionMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		labels.as_slice().chunks(n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			let test_metrics = tangram_metrics::RegressionMetrics::default();
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			model.predict(features, state.predictions.slice_mut(slice));
			state
				.test_metrics
				.update(tangram_metrics::RegressionMetricsInput {
					predictions: state.predictions.slice(slice).as_slice().unwrap(),
					labels,
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_regressor(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::Regressor,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		test_metrics: tangram_metrics::RegressionMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		labels.as_slice().chunks(n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			let test_metrics = tangram_metrics::RegressionMetrics::default();
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			model.predict(features, state.predictions.slice_mut(slice));
			state
				.test_metrics
				.update(tangram_metrics::RegressionMetricsInput {
					predictions: state.predictions.slice(slice).as_slice().unwrap(),
					labels,
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_linear_binary_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::BinaryClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::BinaryClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		test_metrics: tangram_metrics::BinaryClassificationMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			State {
				predictions,
				test_metrics: tangram_metrics::BinaryClassificationMetrics::new(99),
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			let mut predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions.view_mut());
			state
				.test_metrics
				.update(tangram_metrics::BinaryClassificationMetricsInput {
					probabilities: predictions.as_slice().unwrap(),
					labels: labels.as_slice().unwrap(),
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_binary_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::BinaryClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::BinaryClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array1<f32>,
		test_metrics: tangram_metrics::BinaryClassificationMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros(n_examples_per_batch);
			State {
				predictions,
				test_metrics: tangram_metrics::BinaryClassificationMetrics::new(99),
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows()];
			let mut predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions.view_mut());
			state
				.test_metrics
				.update(tangram_metrics::BinaryClassificationMetricsInput {
					probabilities: predictions.as_slice().unwrap(),
					labels: labels.as_slice().unwrap(),
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_linear_multiclass_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_linear::MulticlassClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_f32(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.variants().len();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array2<f32>,
		test_metrics: tangram_metrics::MulticlassClassificationMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros((n_examples_per_batch, n_classes));
			let test_metrics = tangram_metrics::MulticlassClassificationMetrics::new(n_classes);
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows(), ..];
			let predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions);
			let predictions = state.predictions.slice(slice);
			let labels = labels.view();
			state
				.test_metrics
				.update(tangram_metrics::MulticlassClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_multiclass_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[tangram_features::FeatureGroup],
	model: &tangram_tree::MulticlassClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> tangram_metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		tangram_features::compute_features_array_value(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test
		.columns()
		.get(target_column_index)
		.unwrap()
		.as_enum()
		.unwrap();
	let n_classes = labels.variants().len();
	let n_examples_per_batch = 256;
	struct State {
		predictions: Array2<f32>,
		test_metrics: tangram_metrics::MulticlassClassificationMetrics,
	}
	let State { test_metrics, .. } = zip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		{
			let predictions = Array::zeros((n_examples_per_batch, n_classes));
			let test_metrics = tangram_metrics::MulticlassClassificationMetrics::new(n_classes);
			State {
				predictions,
				test_metrics,
			}
		},
		|mut state, (features, labels)| {
			let slice = s![0..features.nrows(), ..];
			let predictions = state.predictions.slice_mut(slice);
			model.predict(features, predictions);
			let predictions = state.predictions.slice(slice);
			let labels = labels.view();
			state
				.test_metrics
				.update(tangram_metrics::MulticlassClassificationMetricsInput {
					probabilities: predictions,
					labels,
				});
			progress_counter.inc(labels.len() as u64);
			state
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}
