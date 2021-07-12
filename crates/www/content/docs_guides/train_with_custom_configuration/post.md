{
"title": "Train with Custom Configuration."
}

The Tangram CLI's default settings are designed to produce good results for most datasets. If you want more control over the training process, you can provide a configuration file. When you run `tangram train`, include your configuration file with `--config path/to/config.json`. The best reference for all the options available is in the source code: [crates/core/config.rs](https://github.com/tangramxyz/tangram/blob/main/crates/core/config.rs).

Below is an example configuration file for the heart disease dataset.

```json
{
	"column_types": {
		"age": {
			"type": "number"
		},
		"chest_pain": {
			"options": [
				"asymptomatic",
				"atypical angina",
				"non-angina pain",
				"typical angina"
			],
			"type": "enum"
		},
		"cholesterol": {
			"type": "number"
		},
		"diagnosis": {
			"options": ["Negative", "Positive"],
			"type": "enum"
		},
		"exercise_induced_angina": {
			"options": ["no", "yes"],
			"type": "enum"
		},
		"exercise_max_heart_rate": {
			"type": "number"
		},
		"exercise_st_depression": {
			"type": "number"
		},
		"exercise_st_slope": {
			"options": ["downsloping", "flat", "upsloping"],
			"type": "enum"
		},
		"fasting_blood_sugar_greater_than_120": {
			"options": ["false", "true"],
			"type": "enum"
		},
		"fluoroscopy_vessels_colored": {
			"options": ["0", "1", "2", "3"],
			"type": "enum"
		},
		"gender": {
			"options": ["female", "male"],
			"type": "enum"
		},
		"resting_blood_pressure": {
			"type": "number"
		},
		"resting_ecg_result": {
			"options": [
				"ST-T wave abnormality",
				"normal",
				"probable or definite left ventricular hypertrophy"
			],
			"type": "enum"
		},
		"thallium_stress_test": {
			"options": ["fixed defect", "normal", "reversible defect"],
			"type": "enum"
		}
	},
	"comparison_fraction": 0.1,
	"comparison_metric": "accuracy",
	"grid": [
		{
			"early_stopping_options": {
				"early_stopping_fraction": 0.1,
				"early_stopping_rounds": 5,
				"early_stopping_threshold": 0.00001
			},
			"l2_regularization": 0.1,
			"learning_rate": 0.1,
			"max_epochs": 100,
			"model": "linear",
			"n_examples_per_batch": 10
		},
		{
			"early_stopping_options": {
				"early_stopping_fraction": 0.1,
				"early_stopping_rounds": 5,
				"early_stopping_threshold": 0.00001
			},
			"l2_regularization_for_continuous_splits": 0.1,
			"l2_regularization_for_discrete_splits": 10,
			"learning_rate": 0.1,
			"max_depth": 50,
			"max_examples_for_computing_bin_thresholds": 200000,
			"max_leaf_nodes": 512,
			"max_rounds": 100,
			"max_valid_bins_for_number_features": 255,
			"min_examples_per_node": 20,
			"min_gain_to_split": 0,
			"min_sum_hessians_per_node": 0.0001,
			"model": "tree",
			"smoothing_factor_for_discrete_bin_sorting": 10
		}
	],
	"shuffle": {
		"seed": 42
	},
	"test_fraction": 0.2
}
```
