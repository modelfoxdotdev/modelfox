use crate::progress::ModelTestProgressEvent;
use modelfox_progress_counter::ProgressCounter;
use modelfox_table::prelude::*;
use modelfox_zip::pzip;
use ndarray::prelude::*;
use rayon::prelude::*;

pub fn test_linear_regressor(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_linear::Regressor,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_f32(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	handle_progress_event(ModelTestProgressEvent::ComputeFeaturesDone);
	let labels = table_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		modelfox_metrics::RegressionMetrics::default,
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros(features.nrows());
			model.predict(features, predictions.view_mut());
			test_metrics.update(modelfox_metrics::RegressionMetricsInput {
				predictions: predictions.as_slice().unwrap(),
				labels: labels.as_slice().unwrap(),
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(
		modelfox_metrics::RegressionMetrics::default,
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_regressor(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_tree::Regressor,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::RegressionMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_value(table_test, feature_groups, &|| {
			progress_counter.inc(1)
		});
	let progress_total = table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::Test(progress_counter.clone()));
	let labels = table_test.columns().get(target_column_index).unwrap();
	let labels = labels.as_number().unwrap();
	let n_examples_per_batch = 256;
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		modelfox_metrics::RegressionMetrics::default,
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros(features.nrows());
			model.predict(features, predictions.view_mut());
			test_metrics.update(modelfox_metrics::RegressionMetricsInput {
				predictions: predictions.as_slice().unwrap(),
				labels: labels.as_slice().unwrap(),
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(
		modelfox_metrics::RegressionMetrics::default,
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_linear_binary_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_linear::BinaryClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::BinaryClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_f32(table_test, feature_groups, &|| {
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
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		|| modelfox_metrics::BinaryClassificationMetrics::new(99),
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros(features.nrows());
			model.predict(features, predictions.view_mut());
			test_metrics.update(modelfox_metrics::BinaryClassificationMetricsInput {
				probabilities: predictions.as_slice().unwrap(),
				labels: labels.as_slice().unwrap(),
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(|| modelfox_metrics::BinaryClassificationMetrics::new(99), {
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		}
	});
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_binary_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_tree::BinaryClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::BinaryClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_value(table_test, feature_groups, &|| {
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
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		|| modelfox_metrics::BinaryClassificationMetrics::new(99),
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros(features.nrows());
			model.predict(features, predictions.view_mut());
			test_metrics.update(modelfox_metrics::BinaryClassificationMetricsInput {
				probabilities: predictions.as_slice().unwrap(),
				labels: labels.as_slice().unwrap(),
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(|| modelfox_metrics::BinaryClassificationMetrics::new(99), {
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		}
	});
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_linear_multiclass_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_linear::MulticlassClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_f32(table_test, feature_groups, &|| {
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
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		|| modelfox_metrics::MulticlassClassificationMetrics::new(n_classes),
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros((features.nrows(), n_classes));
			model.predict(features, predictions.view_mut());
			let labels = labels.view();
			test_metrics.update(modelfox_metrics::MulticlassClassificationMetricsInput {
				probabilities: predictions.view(),
				labels,
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(
		|| modelfox_metrics::MulticlassClassificationMetrics::new(n_classes),
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}

pub fn test_tree_multiclass_classifier(
	table_test: &TableView,
	target_column_index: usize,
	feature_groups: &[modelfox_features::FeatureGroup],
	model: &modelfox_tree::MulticlassClassifier,
	handle_progress_event: &mut dyn FnMut(ModelTestProgressEvent),
) -> modelfox_metrics::MulticlassClassificationMetricsOutput {
	let n_features = feature_groups.iter().map(|g| g.n_features()).sum::<usize>();
	let progress_total = n_features as u64 * table_test.nrows() as u64;
	let progress_counter = ProgressCounter::new(progress_total);
	handle_progress_event(ModelTestProgressEvent::ComputeFeatures(
		progress_counter.clone(),
	));
	let features =
		modelfox_features::compute_features_array_value(table_test, feature_groups, &|| {
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
	let test_metrics = pzip!(
		features.axis_chunks_iter(Axis(0), n_examples_per_batch),
		ArrayView1::from(labels.as_slice()).axis_chunks_iter(Axis(0), n_examples_per_batch),
	)
	.fold(
		|| modelfox_metrics::MulticlassClassificationMetrics::new(n_classes),
		|mut test_metrics, (features, labels)| {
			let mut predictions = Array::zeros((features.nrows(), n_classes));
			model.predict(features, predictions.view_mut());
			let labels = labels.view();
			test_metrics.update(modelfox_metrics::MulticlassClassificationMetricsInput {
				probabilities: predictions.view(),
				labels,
			});
			progress_counter.inc(labels.len() as u64);
			test_metrics
		},
	)
	.reduce(
		|| modelfox_metrics::MulticlassClassificationMetrics::new(n_classes),
		|mut metrics_a, metrics_b| {
			metrics_a.merge(metrics_b);
			metrics_a
		},
	);
	let test_metrics = test_metrics.finalize();
	handle_progress_event(ModelTestProgressEvent::TestDone);
	test_metrics
}
