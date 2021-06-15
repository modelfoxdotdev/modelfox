use pinwheel::prelude::*;
use tangram_charts::{
	components::FeatureContributionsChart,
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_ui as ui;

pub struct ProductionExplanations;

impl Component for ProductionExplanations {
	fn into_node(self) -> Node {
		let class_name = "Positive".to_owned();
		let probability = 0.9748272;
		let series = feature_contributions_chart_series();
		let title = div()
			.class("index-step-title")
			.child("Monitor production predictions.");
		let p1 = div()
			.class("index-step-text")
			.child("After calling ")
			.child(ui::InlineCode::new("logPrediction"))
			.child(", look up any prediction in the app by its identifier.");
		let p2 = "Every prediction will display its input and output, as well as a detailed explanation showing how each feature contributed to the output.";
		let p2 = div().class("index-step-text").child(p2);
		let left = div().child(title).child(p1).child(br()).child(p2);
		let prediction = div()
			.style(style::GRID_AREA, "prediction")
			.child(ui::NumberCard::new("Prediction".to_owned(), class_name));
		let probability = div()
			.style(style::GRID_AREA, "probability")
			.child(ui::NumberCard::new(
				"Probability".to_owned(),
				ui::format_percent(probability),
			));
		let feature_contributions = div()
			.style(style::GRID_AREA, "feature-contributions")
			.child(
				ui::Card::new().child(Dehydrate::new(
					"production-explanations",
					FeatureContributionsChart::new()
						.series(series)
						.negative_color(ui::colors::RED.to_owned())
						.positive_color(ui::colors::GREEN.to_owned())
						.include_x_axis_title(Some(true)),
				)),
			);
		let right = ui::Window::new().child(
			div()
				.class("production-explanations-grid")
				.child(prediction)
				.child(probability)
				.child(feature_contributions),
		);
		div()
			.class("index-step")
			.child(left)
			.child(right)
			.into_node()
	}
}

fn feature_contributions_chart_series() -> Vec<FeatureContributionsChartSeries> {
	vec![FeatureContributionsChartSeries {
		title: "".to_owned(),
		baseline: 0.02783647,
		baseline_label: "51%".to_owned(),
		output: 3.6564934,
		output_label: "97%".to_owned(),
		values: vec![
			FeatureContributionsChartValue {
				feature: "thallium_stress_test = 'normal'".to_owned(),
				value: -4.0369789,
			},
			FeatureContributionsChartValue {
				feature: "resting_blood_pressure = '160'".to_owned(),
				value: -0.0,
			},
			FeatureContributionsChartValue {
				feature: "gender = 'male'".to_owned(),
				value: 2.628391,
			},
			FeatureContributionsChartValue {
				feature: "fasting_blood_sugar_greater_than_120 = 'false'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "resting_ecg_result = 'probable or definite left ventricular hypertrophy'"
					.to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "exercise_induced_angina = 'yes'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "fluoroscopy_vessels_colored = '3'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "age = '67'".to_owned(),
				value: 1.95109127,
			},
			FeatureContributionsChartValue {
				feature: "exercise_st_slope = 'flat'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "exercise_st_depression = '1.5'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "chest_pain = 'asymptomatic'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "exercise_max_heart_rate = '108'".to_owned(),
				value: 0.0,
			},
			FeatureContributionsChartValue {
				feature: "cholesterol = '286'".to_owned(),
				value: 2.9861533,
			},
		],
		..Default::default()
	}]
}
