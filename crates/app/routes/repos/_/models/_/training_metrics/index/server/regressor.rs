use pinwheel::prelude::*;
use tangram_app_ui::colors::{BASELINE_COLOR, TRAINING_COLOR};
use tangram_ui as ui;

pub struct Regressor {
	pub warning: Option<String>,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
}

impl Component for Regressor {
	fn into_node(self) -> Node {
		let rmse_description = "The Root Mean Squared Error (RMSE) is the square root of the mean of the squared differences between each of the predicted values and their corresponding actual value. A perfect model has a RMSE of 0 because it always predicts the correct value.";
		let mse_description = "The Mean Squared Error (MSE) is the mean of the squared differences between each of the predicted values and their corresponding actual value. A perfect model has a MSE of 0 because it always predictions the correct value.";
		ui::S1::new()
			.child(ui::H1::new("Training Metrics"))
			.child(
				ui::S2::new()
					.child(ui::P::new().child(rmse_description))
					.child(
						ui::NumberComparisonCard::new(Some(self.baseline_rmse), Some(self.rmse))
							.color_a(BASELINE_COLOR.to_owned())
							.color_b(TRAINING_COLOR.to_owned())
							.title("Root Mean Squared Error".to_owned())
							.value_a_title("Baseline".to_owned())
							.value_b_title("Training".to_owned())
							.number_formatter(ui::NumberFormatter::float_default()),
					)
					.child(ui::P::new().child(mse_description))
					.child(
						ui::NumberComparisonCard::new(Some(self.baseline_mse), Some(self.mse))
							.color_a(BASELINE_COLOR.to_owned())
							.color_b(TRAINING_COLOR.to_owned())
							.title("Mean Squared Error".to_owned())
							.value_a_title("Baseline".to_owned())
							.value_b_title("Training".to_owned())
							.number_formatter(ui::NumberFormatter::float_default()),
					),
			)
			.into_node()
	}
}
