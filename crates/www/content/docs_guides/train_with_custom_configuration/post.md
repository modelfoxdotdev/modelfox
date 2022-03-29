{
"title": "Train with Custom Configuration."
}

The ModelFox CLI's default settings are designed to produce good results for most datasets. If you want more control over the training process, you can provide a configuration file. When you run `modelfox train`, include your configuration file with `--config path/to/config.json`. The best reference for all the options available is in the source code: [crates/core/config.rs](https://github.com/modelfoxdotdev/modelfox/blob/main/crates/core/config.rs).

Below is an example configuration file for the heart disease dataset.

```json
{
	"dataset": {
		"columns": [
			{
				"name": "age",
				"type": "number"
			},
			{
				"name": "chest_pain",
				"type": "enum",
				"variants": [
					"asymptomatic",
					"atypical angina",
					"non-angina pain",
					"typical angina"
				]
			},
			{
				"name": "cholesterol",
				"type": "number"
			},
			{
				"name": "diagnosis",
				"type": "enum",
				"variants": ["Negative", "Positive"]
			},
			{
				"name": "exercise_induced_angina",
				"type": "enum",
				"variants": ["no", "yes"]
			},
			{
				"name": "exercise_max_heart_rate",
				"type": "number"
			},
			{
				"name": "exercise_max_heart_rate",
				"type": "number"
			},
			{
				"name": "exercise_st_slope",
				"type": "enum",
				"variants": ["downsloping", "flat", "upsloping"]
			},
			{
				"name": "fasting_blood_sugar_greater_than_120",
				"type": "enum",
				"variants": ["false", "true"]
			},
			{
				"name": "fluoroscopy_vessels_colored",
				"type": "enum",
				"variants": ["0", "1", "2", "3"]
			},
			{
				"name": "gender",
				"type": "enum",
				"variants": ["female", "male"]
			},
			{
				"name": "resting_blood_pressure",
				"type": "number"
			},
			{
				"name": "resting_ecg_result",
				"type": "enum",
				"variants": [
					"normal",
					"probable or definite left ventricular hypertrophy",
					"ST-T wave abnormality"
				]
			},
			{
				"name": "thallium_stress_test",
				"type": "enum",
				"variants": ["normal", "reversible defect", "fixed defect"]
			}
		],
		"comparison_fraction": 0.1,
		"test_fraction": 0.2
	},
	"train": {
		"grid": [
			{
				"learning_rate": 0.1,
				"max_leaf_nodes": 1000,
				"max_rounds": 1000,
				"max_valid_bins_for_number_features": 255,
				"min_examples_per_node": 2,
				"min_gain_to_split": 0,
				"min_sum_hessians_per_node": 0.001,
				"model": "tree",
				"smoothing_factor_for_discrete_bin_sorting": 10
			}
		]
	}
}
```
