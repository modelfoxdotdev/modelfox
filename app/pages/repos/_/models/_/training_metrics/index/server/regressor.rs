use html::{component, html, Props};
use tangram_app_common::tokens::{BASELINE_COLOR, TRAINING_COLOR};
use tangram_ui as ui;

#[derive(Props)]
pub struct RegressorProps {
	pub warning: Option<String>,
	pub baseline_mse: f32,
	pub baseline_rmse: f32,
	pub mse: f32,
	pub rmse: f32,
}

#[component]
pub fn Regressor(props: RegressorProps) {
	let rmse_description = "The Root Mean Squared Error (RMSE) is the square root of the mean of the squared differences between each of the predicted values and their corresponding actual value. A perfect model has a RMSE of 0 because it always predicts the correct value.";
	let mse_description = "The Mean Squared Error (MSE) is the mean of the squared differences between each of the predicted values and their corresponding actual value. A perfect model has a MSE of 0 because it always predictions the correct value.";
	html! {
		<ui::S1>
			<ui::H1>{"Training Metrics"}</ui::H1>
			<ui::S2>
				<ui::P>
					{rmse_description}
				</ui::P>
				<ui::NumberComparisonCard
					color_a={Some(BASELINE_COLOR.to_owned())}
					color_b={Some(TRAINING_COLOR.to_owned())}
					title="Root Mean Squared Error"
					value_a={Some(props.baseline_rmse)}
					value_a_title="Baseline"
					value_b={Some(props.rmse)}
					value_b_title="Training"
					number_formatter={ui::NumberFormatter::float_default()}
				/>
				<ui::P>
					{mse_description}
				</ui::P>
				<ui::NumberComparisonCard
					color_a={Some(BASELINE_COLOR.to_owned())}
					color_b={Some(TRAINING_COLOR.to_owned())}
					title="Mean Squared Error"
					value_a={Some(props.baseline_mse)}
					value_a_title="Baseline"
					value_b={Some(props.mse)}
					value_b_title="Training"
					number_formatter={ui::NumberFormatter::float_default()}
				/>
			</ui::S2>
		</ui::S1>
	}
}
