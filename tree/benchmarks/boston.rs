use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_table::prelude::*;
use tangram_tree::Progress;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/boston_train.csv");
	let csv_file_path_test = Path::new("data/boston_test.csv");
	let target_column_index = 13;
	let mut features_train =
		Table::from_path(csv_file_path_train, Default::default(), &mut |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let mut features_test =
		Table::from_path(csv_file_path_test, Default::default(), &mut |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_number().unwrap();
	let labels_test = labels_test.as_number().unwrap();

	// Train the model.
	let train_output = tangram_tree::Regressor::train(
		features_train.view(),
		labels_train.view(),
		&tangram_tree::TrainOptions {
			learning_rate: 0.1,
			max_leaf_nodes: 255,
			max_rounds: 100,
			..Default::default()
		},
		Progress {
			kill_chip: &tangram_kill_chip::KillChip::default(),
			handle_progress_event: &mut |_| {},
		},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut predictions = Array::zeros(labels_test.len());
	train_output
		.model
		.predict(features_test.view(), predictions.view_mut());

	// Compute metrics.
	let mut metrics = tangram_metrics::RegressionMetrics::new();
	metrics.update(tangram_metrics::RegressionMetricsInput {
		predictions: predictions.as_slice().unwrap(),
		labels: labels_test.view().as_slice(),
	});
	let metrics = metrics.finalize();

	let output = json!({
		"mse": metrics.mse,
	});
	println!("{}", output);
}
