use modelfox_linear::Progress;
use modelfox_table::prelude::*;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/boston_train.csv");
	let csv_file_path_test = Path::new("data/boston_test.csv");
	let target_column_index = 13;
	let mut features_train =
		Table::from_path(csv_file_path_train, Default::default(), &mut |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_number().unwrap();
	let mut features_test =
		Table::from_path(csv_file_path_test, Default::default(), &mut |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_number().unwrap();
	let feature_groups: Vec<modelfox_features::FeatureGroup> = features_train
		.columns()
		.iter()
		.map(|column| {
			modelfox_features::FeatureGroup::Normalized(
				modelfox_features::NormalizedFeatureGroup::compute_for_column(column.view()),
			)
		})
		.collect();
	let features_train = modelfox_features::compute_features_array_f32(
		&features_train.view(),
		feature_groups.as_slice(),
		&|| {},
	);
	let features_test = modelfox_features::compute_features_array_f32(
		&features_test.view(),
		feature_groups.as_slice(),
		&|| {},
	);

	// Train the model.
	let train_output = modelfox_linear::Regressor::train(
		features_train.view(),
		labels_train.view(),
		&modelfox_linear::TrainOptions {
			learning_rate: 0.01,
			max_epochs: 1,
			n_examples_per_batch: 1,
			..Default::default()
		},
		Progress {
			kill_chip: &modelfox_kill_chip::KillChip::default(),
			handle_progress_event: &mut |_| {},
		},
	);

	// Make predictions on the test data.
	let mut predictions = Array::zeros(labels_test.len());
	train_output
		.model
		.predict(features_test.view(), predictions.view_mut());

	// Compute metrics.
	let mut metrics = modelfox_metrics::RegressionMetrics::new();
	metrics.update(modelfox_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.view().as_slice(),
	});
	let metrics = metrics.finalize();
	let output = json!({"mse": metrics.mse});
	println!("{}", output);
}
