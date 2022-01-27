use modelfox_table::prelude::*;
use modelfox_tree::Progress;
use modelfox_zip::zip;
use ndarray::prelude::*;
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/heart_disease_train.csv");
	let csv_file_path_test = Path::new("data/heart_disease_test.csv");
	let _n_rows_train = 242;
	let n_rows_test = 61;
	let target_column_index = 13;
	let gender_variants = ["male", "female"].iter().map(ToString::to_string).collect();
	let chest_pain_variants = [
		"typical angina",
		"asymptomatic",
		"non-angina pain",
		"atypical angina",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let fasting_blood_sugar_greater_than_120_variants =
		["True", "False"].iter().map(ToString::to_string).collect();
	let resting_ecg_result_variants = [
		"probable or definite left ventricular hypertrophy",
		"normal",
		"ST-T wave abnormality",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let exercise_induced_angina_variants = ["no", "yes"].iter().map(ToString::to_string).collect();
	let exercise_st_slope_variants = ["downsloping", "flat", "upsloping"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let thallium_stress_test_variants = ["fixed defect", "normal", "reversible defect"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let fluoroscopy_vessels_colored_variants = ["0", "1", "2", "3"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let diagnosis_variants = ["Negative", "Positive"]
		.iter()
		.map(ToString::to_string)
		.collect();
	let options = modelfox_table::Options {
		column_types: Some(BTreeMap::from([
			("age".to_owned(), TableColumnType::Number),
			(
				"gender".to_owned(),
				TableColumnType::Enum {
					variants: gender_variants,
				},
			),
			(
				"chest_pain".to_owned(),
				TableColumnType::Enum {
					variants: chest_pain_variants,
				},
			),
			("resting_blood_pressure".to_owned(), TableColumnType::Number),
			("cholesterol".to_owned(), TableColumnType::Number),
			(
				"fasting_blood_sugar_greater_than_120".to_owned(),
				TableColumnType::Enum {
					variants: fasting_blood_sugar_greater_than_120_variants,
				},
			),
			(
				"resting_ecg_result".to_owned(),
				TableColumnType::Enum {
					variants: resting_ecg_result_variants,
				},
			),
			(
				"exercise_max_heart_rate".to_owned(),
				TableColumnType::Number,
			),
			(
				"exercise_induced_angina".to_owned(),
				TableColumnType::Enum {
					variants: exercise_induced_angina_variants,
				},
			),
			("exercise_st_depression".to_owned(), TableColumnType::Number),
			(
				"exercise_st_slope".to_owned(),
				TableColumnType::Enum {
					variants: exercise_st_slope_variants,
				},
			),
			(
				"fluoroscopy_vessels_colored".to_owned(),
				TableColumnType::Enum {
					variants: fluoroscopy_vessels_colored_variants,
				},
			),
			(
				"thallium_stress_test".to_owned(),
				TableColumnType::Enum {
					variants: thallium_stress_test_variants,
				},
			),
			(
				"diagnosis".to_owned(),
				TableColumnType::Enum {
					variants: diagnosis_variants,
				},
			),
		])),
		..Default::default()
	};
	let mut features_train =
		Table::from_path(csv_file_path_train, options.clone(), &mut |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let mut features_test =
		Table::from_path(csv_file_path_test, options.clone(), &mut |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let train_output = modelfox_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&modelfox_tree::TrainOptions {
			max_leaf_nodes: 255,
			..Default::default()
		},
		Progress {
			kill_chip: &modelfox_kill_chip::KillChip::default(),
			handle_progress_event: &mut |_| {},
		},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let mut probabilities = Array::zeros(n_rows_test);
	train_output
		.model
		.predict(features_test.view(), probabilities.view_mut());

	// Compute metrics.
	let input = zip!(probabilities.iter(), labels_test.iter())
		.map(|(probability, label)| (*probability, label.unwrap()))
		.collect();
	let auc_roc = modelfox_metrics::AucRoc::compute(input);

	let output = json!({
		"auc_roc": auc_roc,
	});
	println!("{}", output);
}
