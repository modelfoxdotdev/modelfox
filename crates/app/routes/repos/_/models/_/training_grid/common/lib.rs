pub fn hyperparameters_for_grid_item(
	train_grid_item_output: &modelfox_model::TrainGridItemOutputReader,
) -> Vec<(String, String)> {
	match &train_grid_item_output.hyperparameters() {
		modelfox_model::ModelTrainOptionsReader::Linear(hyperparameters) => {
			let hyperparameters = hyperparameters.read();
			vec![
				(
					"l2_regularization".to_owned(),
					hyperparameters.l2_regularization().to_string(),
				),
				(
					"learning_rate".to_owned(),
					hyperparameters.learning_rate().to_string(),
				),
				(
					"max_epochs".to_owned(),
					hyperparameters.max_epochs().to_string(),
				),
				(
					"n_examples_per_batch".to_owned(),
					hyperparameters.n_examples_per_batch().to_string(),
				),
				(
					"early_stopping:early_stopping_fraction".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| options.early_stopping_fraction().to_string())
						.unwrap_or_else(|| "None".to_string()),
				),
				(
					"early_stopping:n_rounds_without_improvement_to_stop".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| options.n_rounds_without_improvement_to_stop().to_string())
						.unwrap_or_else(|| "None".to_string()),
				),
				(
					"early_stopping:min_decrease_in_loss_for_significant_change".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| {
							options
								.min_decrease_in_loss_for_significant_change()
								.to_string()
						})
						.unwrap_or_else(|| "None".to_string()),
				),
			]
		}
		modelfox_model::ModelTrainOptionsReader::Tree(hyperparameters) => {
			let hyperparameters = hyperparameters.read();
			vec![
				(
					"binned_features_layout".to_owned(),
					match hyperparameters.binned_features_layout() {
						modelfox_model::BinnedFeaturesLayoutReader::RowMajor(_) => {
							"row major".to_owned()
						}
						modelfox_model::BinnedFeaturesLayoutReader::ColumnMajor(_) => {
							"column major".to_owned()
						}
					},
				),
				(
					"early_stopping:early_stopping_fraction".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| options.early_stopping_fraction().to_string())
						.unwrap_or_else(|| "None".to_string()),
				),
				(
					"early_stopping:n_rounds_without_improvement_to_stop".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| options.n_rounds_without_improvement_to_stop().to_string())
						.unwrap_or_else(|| "None".to_string()),
				),
				(
					"early_stopping:min_decrease_in_loss_for_significant_change".to_owned(),
					hyperparameters
						.early_stopping_options()
						.map(|options| {
							options
								.min_decrease_in_loss_for_significant_change()
								.to_string()
						})
						.unwrap_or_else(|| "None".to_string()),
				),
				(
					"l2_regularization_for_continuous_splits".to_owned(),
					hyperparameters
						.l2_regularization_for_continuous_splits()
						.to_string(),
				),
				(
					"l2_regularization_for_discrete_splits".to_owned(),
					hyperparameters
						.l2_regularization_for_discrete_splits()
						.to_string(),
				),
				(
					"learning_rate".to_owned(),
					hyperparameters.learning_rate().to_string(),
				),
				(
					"max_depth".to_owned(),
					hyperparameters
						.max_depth()
						.map(|max_depth| max_depth.to_string())
						.unwrap_or_else(|| "None".to_owned()),
				),
				(
					"max_examples_for_computing_bin_thresholds".to_owned(),
					hyperparameters
						.max_examples_for_computing_bin_thresholds()
						.to_string(),
				),
				(
					"max_leaf_nodes".to_owned(),
					hyperparameters.max_leaf_nodes().to_string(),
				),
				(
					"max_rounds".to_owned(),
					hyperparameters.max_rounds().to_string(),
				),
				(
					"max_valid_bins_for_number_features".to_owned(),
					hyperparameters
						.max_valid_bins_for_number_features()
						.to_string(),
				),
				(
					"min_examples_per_node".to_owned(),
					hyperparameters.min_examples_per_node().to_string(),
				),
				(
					"min_gain_to_split".to_owned(),
					hyperparameters.min_gain_to_split().to_string(),
				),
				(
					"min_sum_hessians_per_node".to_owned(),
					hyperparameters.min_sum_hessians_per_node().to_string(),
				),
				(
					"smoothing_factor_for_discrete_bin_sorting".to_owned(),
					hyperparameters
						.smoothing_factor_for_discrete_bin_sorting()
						.to_string(),
				),
			]
		}
	}
}
