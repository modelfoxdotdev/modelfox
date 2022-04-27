use modelfox_table::prelude::*;
use modelfox_tree::Progress;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/iris_train.csv");
	let csv_file_path_test = Path::new("data/iris_test.csv");
	let _n_rows_train = 120;
	let n_rows_test = 30;
	let target_column_index = 4;
	let n_classes = 3;
	let mut features_train =
		Table::from_path(csv_file_path_train, Default::default(), &mut |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let mut features_test =
		Table::from_path(csv_file_path_test, Default::default(), &mut |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let train_output = modelfox_tree::MulticlassClassifier::train(
		features_train.view(),
		labels_train.view(),
		&Default::default(),
		Progress {
			kill_chip: &modelfox_kill_chip::KillChip::default(),
			handle_progress_event: &mut |_| {},
		},
	);

	// Make predictions on the test data.
	let mut probabilities = Array::zeros((n_rows_test, 3));
	let features_test = features_test.to_rows();
	train_output
		.model
		.predict(features_test.view(), probabilities.view_mut());

	// Compute Metrics.
	let mut metrics = modelfox_metrics::MulticlassClassificationMetrics::new(n_classes);
	metrics.update(modelfox_metrics::MulticlassClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels_test.view().as_slice().into(),
	});
	let metrics = metrics.finalize();

	let output = json!({
		"accuracy": metrics.accuracy,
	});
	println!("{}", output);
}
