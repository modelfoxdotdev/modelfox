use html::Props;

#[derive(Props, serde::Serialize, serde::Deserialize)]
pub struct ClientProps {
	pub threshold_metrics: Vec<Metrics>,
	pub baseline_metrics: Metrics,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Metrics {
	pub accuracy: f32,
	pub f1_score: Option<f32>,
	pub false_negatives_fraction: f32,
	pub false_positives_fraction: f32,
	pub precision: Option<f32>,
	pub recall: Option<f32>,
	pub threshold: f32,
	pub true_negatives_fraction: f32,
	pub true_positives_fraction: f32,
}
