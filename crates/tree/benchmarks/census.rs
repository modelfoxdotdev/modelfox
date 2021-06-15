use maplit::btreemap;
use ndarray::prelude::*;
use serde_json::json;
use std::path::Path;
use tangram_table::prelude::*;
use tangram_tree::Progress;
use tangram_zip::zip;

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/census_train.csv");
	let csv_file_path_test = Path::new("data/census_test.csv");
	let _n_rows_train = 26049;
	let n_rows_test = 6512;
	let target_column_index = 14;
	let workclass_variants = [
		"State-gov",
		"Self-emp-not-inc",
		"Private",
		"Federal-gov",
		"Local-gov",
		"?",
		"Self-emp-inc",
		"Without-pay",
		"Never-worked",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let education_variants = [
		"Bachelors",
		"HS-grad",
		"11th",
		"Masters",
		"9th",
		"Some-college",
		"Assoc-acdm",
		"Assoc-voc",
		"7th-8th",
		"Doctorate",
		"Prof-school",
		"5th-6th",
		"10th",
		"1st-4th",
		"Preschool",
		"12th",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let marital_status_variants = [
		"Never-married",
		"Married-civ-spouse",
		"Divorced",
		"Married-spouse-absent",
		"Separated",
		"Married-AF-spouse",
		"Widowed",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let occupation_variants = [
		"Adm-clerical",
		"Exec-managerial",
		"Handlers-cleaners",
		"Prof-specialty",
		"Other-service",
		"Sales",
		"Craft-repair",
		"Transport-moving",
		"Farming-fishing",
		"Machine-op-inspct",
		"Tech-support",
		"?",
		"Protective-serv",
		"Armed-Forces",
		"Priv-house-serv",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let relationship_variants = [
		"Not-in-family",
		"Husband",
		"Wife",
		"Own-child",
		"Unmarried",
		"Other-relative",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let race_variants = [
		"White",
		"Black",
		"Asian-Pac-Islander",
		"Amer-Indian-Eskimo",
		"Other",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let sex_variants = ["Male", "Female"].iter().map(ToString::to_string).collect();
	let native_country_variants = [
		"United-States",
		"Cuba",
		"Jamaica",
		"India",
		"?",
		"Mexico",
		"South",
		"Puerto-Rico",
		"Honduras",
		"England",
		"Canada",
		"Germany",
		"Iran",
		"Philippines",
		"Italy",
		"Poland",
		"Columbia",
		"Cambodia",
		"Thailand",
		"Ecuador",
		"Laos",
		"Taiwan",
		"Haiti",
		"Portugal",
		"Dominican-Republic",
		"El-Salvador",
		"France",
		"Guatemala",
		"China",
		"Japan",
		"Yugoslavia",
		"Peru",
		"Outlying-US(Guam-USVI-etc)",
		"Scotland",
		"Trinadad&Tobago",
		"Greece",
		"Nicaragua",
		"Vietnam",
		"Hong",
		"Ireland",
		"Hungary",
		"Holand-Netherlands",
	]
	.iter()
	.map(ToString::to_string)
	.collect();
	let income_variants = ["<=50K", ">50K"].iter().map(ToString::to_string).collect();
	let options = tangram_table::FromCsvOptions {
		column_types: Some(btreemap!(
		  "age".to_owned() => TableColumnType::Number ,
			"workclass".to_owned() => TableColumnType::Enum { variants: workclass_variants },
			"fnlwgt".to_owned() => TableColumnType::Number,
			"education".to_owned() => TableColumnType::Enum { variants: education_variants },
			"education_num".to_owned() => TableColumnType::Number,
			"marital_status".to_owned() => TableColumnType::Enum { variants: marital_status_variants },
			"occupation".to_owned() => TableColumnType::Enum { variants: occupation_variants },
			"relationship".to_owned() => TableColumnType::Enum { variants: relationship_variants },
			"race".to_owned() => TableColumnType::Enum { variants: race_variants },
			"sex".to_owned() => TableColumnType::Enum { variants: sex_variants },
			"capital_gain".to_owned() => TableColumnType::Number,
			"capital_loss".to_owned() => TableColumnType::Number,
			"hours_per_week".to_owned() => TableColumnType::Number,
			"native_country".to_owned() => TableColumnType::Enum { variants: native_country_variants },
			"income".to_owned() => TableColumnType::Enum { variants: income_variants },
		)),
		..Default::default()
	};
	let mut features_train =
		Table::from_path(csv_file_path_train, options.clone(), &mut |_| {}).unwrap();
	let labels_train = features_train.columns_mut().remove(target_column_index);
	let labels_train = labels_train.as_enum().unwrap();
	let mut features_test =
		Table::from_path(csv_file_path_test, options.clone(), &mut |_| {}).unwrap();
	let labels_test = features_test.columns_mut().remove(target_column_index);
	let labels_test = labels_test.as_enum().unwrap();

	// Train the model.
	let train_options = tangram_tree::TrainOptions {
		learning_rate: 0.1,
		max_leaf_nodes: 255,
		max_rounds: 100,
		..Default::default()
	};
	let train_output = tangram_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&train_options,
		Progress {
			kill_chip: &tangram_kill_chip::KillChip::default(),
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
	let auc_roc = tangram_metrics::AucRoc::compute(input);

	let output = json!({
		"auc_roc": auc_roc,
	});
	println!("{}", output);
}
