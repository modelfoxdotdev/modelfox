use html::{component, html, style};
use tangram_charts::{
	components::FeatureContributionsChart,
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_ui as ui;

#[component]
pub fn ProductionExplanations() {
	let class_name = "Positive".to_owned();
	let probability = 0.9748272;
	let series = feature_contributions_chart_series();
	html! {
		<div class="index-step">
			<div>
				<div class="index-step-title">{"Monitor production predictions."}</div>
				<div class="index-step-text">
					{"After calling "}
					<ui::InlineCode>{"logPrediction"}</ui::InlineCode>
					{", look up any prediction in the app by its identifier."}
				</div>
				<br />
				<div class="index-step-text">
					{"Every prediction will display its input and output, as well as a detailed explanation showing how each feature contributed to the output."}
				</div>
			</div>
			<ui::Window padding={Some(true)}>
				<div class="production-explanations-grid">
					<div style={style! { "grid-area" => "prediction" }}>
						<ui::NumberCard
							title="Prediction"
							value={class_name}
						/>
					</div>
					<div style={style! { "grid-area" => "probability" }}>
						<ui::NumberCard
							title="Probability"
							value={ui::format_percent(probability)}
						/>
					</div>
					<div style={style! { "grid-area" => "feature_contributions" }}>
						<ui::Card>
							<FeatureContributionsChart
								id?="production-explanations"
								include_x_axis_title?={Some(true)}
								negative_color={ui::colors::RED.to_owned()}
								positive_color={ui::colors::GREEN.to_owned()}
								series={series}
							/>
						</ui::Card>
					</div>
				</div>
			</ui::Window>
		</div>
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
