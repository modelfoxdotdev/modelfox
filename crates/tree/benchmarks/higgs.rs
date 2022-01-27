use modelfox_table::prelude::*;
use modelfox_tree::Progress;
use modelfox_zip::{pzip, zip};
use ndarray::prelude::*;
use rayon::prelude::*;
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

fn main() {
	// Load the data.
	let csv_file_path_train = Path::new("data/higgs_train.csv");
	let csv_file_path_test = Path::new("data/higgs_test.csv");
	let target_column_index = 0;
	let signal_variants = ["false", "true"].iter().map(ToString::to_string).collect();
	let options = modelfox_table::Options {
		column_types: Some(BTreeMap::from([
			(
				"signal".to_owned(),
				TableColumnType::Enum {
					variants: signal_variants,
				},
			),
			("lepton_pt".to_owned(), TableColumnType::Number),
			("lepton_eta".to_owned(), TableColumnType::Number),
			("lepton_phi".to_owned(), TableColumnType::Number),
			(
				"missing_energy_magnitude".to_owned(),
				TableColumnType::Number,
			),
			("missing_energy_phi".to_owned(), TableColumnType::Number),
			("jet_1_pt".to_owned(), TableColumnType::Number),
			("jet_1_eta".to_owned(), TableColumnType::Number),
			("jet_1_phi".to_owned(), TableColumnType::Number),
			("jet_1_b_tag".to_owned(), TableColumnType::Number),
			("jet_2_pt".to_owned(), TableColumnType::Number),
			("jet_2_eta".to_owned(), TableColumnType::Number),
			("jet_2_phi".to_owned(), TableColumnType::Number),
			("jet_2_b_tag".to_owned(), TableColumnType::Number),
			("jet_3_pt".to_owned(), TableColumnType::Number),
			("jet_3_eta".to_owned(), TableColumnType::Number),
			("jet_3_phi".to_owned(), TableColumnType::Number),
			("jet_3_b_tag".to_owned(), TableColumnType::Number),
			("jet_4_pt".to_owned(), TableColumnType::Number),
			("jet_4_eta".to_owned(), TableColumnType::Number),
			("jet_4_phi".to_owned(), TableColumnType::Number),
			("jet_4_b_tag".to_owned(), TableColumnType::Number),
			("m_jj".to_owned(), TableColumnType::Number),
			("m_jjj".to_owned(), TableColumnType::Number),
			("m_lv".to_owned(), TableColumnType::Number),
			("m_jlv".to_owned(), TableColumnType::Number),
			("m_bb".to_owned(), TableColumnType::Number),
			("m_wbb".to_owned(), TableColumnType::Number),
			("m_wwbb".to_owned(), TableColumnType::Number),
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
	let train_options = modelfox_tree::TrainOptions {
		binned_features_layout: modelfox_tree::BinnedFeaturesLayout::RowMajor,
		learning_rate: 0.1,
		max_leaf_nodes: 255,
		max_rounds: 100,
		..Default::default()
	};
	let train_output = modelfox_tree::BinaryClassifier::train(
		features_train.view(),
		labels_train.view(),
		&train_options,
		Progress {
			kill_chip: &modelfox_kill_chip::KillChip::default(),
			handle_progress_event: &mut |_| {},
		},
	);

	// Make predictions on the test data.
	let features_test = features_test.to_rows();
	let chunk_size =
		(features_test.nrows() + rayon::current_num_threads() - 1) / rayon::current_num_threads();
	let mut probabilities = Array::zeros(features_test.nrows());
	pzip!(
		features_test.axis_chunks_iter(Axis(0), chunk_size),
		probabilities.axis_chunks_iter_mut(Axis(0), chunk_size),
	)
	.for_each(|(features_test_chunk, probabilities_chunk)| {
		train_output
			.model
			.predict(features_test_chunk, probabilities_chunk);
	});

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
