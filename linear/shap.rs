use ndarray::prelude::*;
use tangram_zip::zip;

pub struct ComputeShapValuesForExampleOutput {
	pub baseline_value: f32,
	pub output_value: f32,
	pub feature_contribution_values: Vec<f32>,
}

/// Compute the SHAP values for a single class for a single example.
pub fn compute_shap_values_for_example(
	features: &[f32],
	bias: f32,
	weights: ArrayView1<f32>,
	means: &[f32],
) -> ComputeShapValuesForExampleOutput {
	let baseline_value = bias
		+ zip!(weights, means)
			.map(|(weight, mean)| weight * mean)
			.sum::<f32>();
	let feature_contributions: Vec<f32> = zip!(weights, features, means)
		.map(|(weight, feature, mean)| weight * (feature - mean))
		.collect();
	let output_value = baseline_value + feature_contributions.iter().sum::<f32>();
	ComputeShapValuesForExampleOutput {
		baseline_value,
		output_value,
		feature_contribution_values: feature_contributions,
	}
}
