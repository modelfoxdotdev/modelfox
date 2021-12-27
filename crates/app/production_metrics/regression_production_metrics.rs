use num::ToPrimitive;
use tangram_app_common::monitor_event::NumberOrString;
use tangram_app_production_stats::NumberStats;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RegressionProductionPredictionMetrics {
	stats: Option<NumberStats>,
	absolute_error: f64,
	squared_error: f64,
}

#[derive(serde::Deserialize)]
pub struct RegressionProductionPredictionMetricsOutput {
	pub mse: f32,
	pub rmse: f32,
	pub mae: f32,
	pub r2: f32,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
}

impl Default for RegressionProductionPredictionMetrics {
	fn default() -> RegressionProductionPredictionMetrics {
		RegressionProductionPredictionMetrics {
			stats: None,
			absolute_error: 0.0,
			squared_error: 0.0,
		}
	}
}

impl RegressionProductionPredictionMetrics {
	pub fn new() -> RegressionProductionPredictionMetrics {
		RegressionProductionPredictionMetrics::default()
	}

	pub fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		let prediction = match value.0.as_number() {
			Ok(value) => value,
			Err(_) => return,
		};
		let label = match value.1.as_number() {
			Ok(value) => value,
			Err(_) => return,
		};
		let absolute_error = prediction - label;
		let squared_error = absolute_error * absolute_error;
		match &mut self.stats {
			Some(stats) => stats.update(prediction),
			None => {
				self.stats.replace(NumberStats::new(prediction));
			}
		};
		self.absolute_error += absolute_error as f64;
		self.squared_error += squared_error as f64;
	}

	pub fn merge(&mut self, other: RegressionProductionPredictionMetrics) {
		match &mut self.stats {
			Some(stats) => {
				if let Some(other) = other.stats {
					stats.merge(other)
				}
			}
			None => self.stats = other.stats,
		};
		self.absolute_error += other.absolute_error;
		self.squared_error += other.squared_error;
	}

	pub fn finalize(self) -> Option<RegressionProductionPredictionMetricsOutput> {
		let stats = self.stats.map(|s| s.finalize());
		match stats {
			Some(stats) => {
				let variance = stats.variance;
				let mae = self.absolute_error.to_f32().unwrap() / stats.n.to_f32().unwrap();
				let mse = self.squared_error.to_f32().unwrap() / stats.n.to_f32().unwrap();
				let rmse = mse.sqrt();
				let r2 = 1.0
					- self.squared_error.to_f32().unwrap() / (variance * stats.n.to_f32().unwrap()); // Sum of Squared Error = variance * n
				let baseline_mse = variance;
				let baseline_rmse = baseline_mse.sqrt();
				Some(RegressionProductionPredictionMetricsOutput {
					mse,
					rmse,
					mae,
					r2,
					baseline_mse,
					baseline_rmse,
				})
			}
			None => None,
		}
	}
}
