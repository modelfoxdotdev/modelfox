use indoc::indoc;
use pinwheel::prelude::*;
use tangram_app_production_stats_column_server::{
	EnumColumnInvalidValuesSection, EnumColumnOverallHistogramEntry, EnumColumnStatsSection,
	EnumColumnUniqueValuesSection, EnumInvalidValuesTable, EnumInvalidValuesTableRow,
	EnumUniqueValuesTable, EnumUniqueValuesTableRow,
};
use tangram_app_production_stats_index_server::{ColumnStatsTable, ColumnStatsTableRow};
use tangram_app_ui::{column_type::ColumnType, date_window::DateWindow};
use tangram_charts::{
	components::FeatureContributionsChart,
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::Document,
};

#[derive(ComponentBuilder)]
pub struct Page {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let p1 = ui::P::new()
			.child("Once our model is deployed, we want to make sure that it performs as well in production as it did in training. We can opt in to logging predictions by calling ")
			.child(ui::InlineCode::new().child("logPrediction"))
			.child(". Later on, as we get official diagnoses for patients, we can call ")
			.child(ui::InlineCode::new().child("logTrueValue"))
			.child(" and use the same identifier as we used in the call to ")
			.child(ui::InlineCode::new().child("logPrediction"))
			.child(".");
		let p2 = ui::P::new().child("Back in the app, we can look up a prediction by its identifier, and get an explanation which shows how each feature affects the output.");
		let p3 = ui::P::new().child("Now let's see how accurate our model has been in production. Lets open the app and view 'Production Metrics'");
		let p4 = ui::P::new().child("Uh oh! It's a bit lower than we expected. Let's try to find the cause. Under 'Production Stats', we see that the 'chest_pain' column has an alert and a high unknown values count. Click on the column to view more details.");
		let p5 = ui::P::new().child("It looks like there is a large discrepancy between the value 'asymptomatic' in production versus training. In the table below, we see a high number of invalid values with the string 'asx'. It looks like we are accidentally using the string 'asx' in our code instead of 'asymptomatic' for the chest pain column. We can update our code to use the correct value and follow the metrics going forward to confirm they bounce back.");
		let p6 = ui::P::new().child("Hooray! We have made it to the end of the getting started guide! In this guide, we learned how to train a model, make predictions from our code, tune our model, and monitor it in production. If you're still unsure of you how want to use Tangram for your own datasets, reach out to us! We are happy to help.");
		let section = ui::S2::new()
			.child(p1)
			.child(Log::new())
			.child(p2)
			.child(ProductionExplanations::new())
			.child(p3)
			.child(ProductionMetrics::new())
			.child(p4)
			.child(ProductionStats::new())
			.child(p5)
			.child(ProductionColumnStats::new())
			.child(p6);
		let buttons = div().class("docs-prev-next-buttons").child(
			ui::Link::new()
				.href("inspect".to_owned())
				.child("< Previous: Inspect your model."),
		);
		Document::new()
			.client("tangram_www_docs_monitor_client")
			.child(
				DocsLayout::new(
					DocsPage::GettingStarted(GettingStartedPage::Monitor),
					Vec::new(),
				)
				.child(
					ui::S1::new()
						.child(ui::H1::new().child("Monitor"))
						.child(section)
						.child(buttons),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct Log {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Log {
	fn into_node(self) -> Node {
		let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
			elixir: indoc!(
				r#"
					# Log the prediction.
					Tangram.log_prediction(model, %Tangram.LogPredictionArgs{
						identifier: "John Doe",
						options: predict_options,
						input: input,
						output: output,
					})

					# Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					Tangram.log_true_value(model, %Tangram.LogTrueValueArgs{
						identifier: "John Doe",
						true_value: "Positive",
					})
				"#
			).into(),
			go: indoc!(
				r#"
					// Log the prediction.
					err = model.LogPrediction(tangram.LogPredictionArgs{
						Identifier: "John Doe",
						Input:      input,
						Options:    predictOptions,
						Output:     output,
					})
					if err != nil {
						log.Fatal(err)
					}

					// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					err = model.LogTrueValue(tangram.LogTrueValueArgs{
						Identifier: "John Doe",
						TrueValue:  "Positive",
					})
					if err != nil {
						log.Fatal(err)
					}
			"#
			).into(),
			javascript: indoc!(
				r#"
					// Log the prediction.
					model.logPrediction({
						identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
						input,
						options,
						output,
					})

					// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					model.logTrueValue({
						identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
						trueValue: "Positive",
					})
				"#
			).into(),
			python: indoc!(
				r#"
					# Log the prediction.
					model.log_prediction(
							identifier="John Doe",
							input=input,
							output=output,
							options=predict_options,
					)

					# Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					model.log_true_value(
							identifier="John Doe",
							true_value="Positive",
					)
				"#
			).into(),
			ruby: indoc!(
				r#"
					# Log the prediction.
					model.log_prediction(
						identifier: 'John Doe',
						input: input,
						output: output,
						options: options
					)

					# Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					model.log_true_value(
						identifier: 'John Doe',
						true_value: 'Positive'
					)
				"#
			).into(),
			rust: indoc!(
				r#"
					// Log the prediction.
					model.log_prediction(tangram::LogPredictionArgs {
						identifier: "John Doe".into(),
						input,
						options: Some(options),
						output,
					})?;

					// Later on, if we get an official diagnosis for the patient, log the true value. Make sure to match the `identifier`.
					model.log_true_value(tangram::LogTrueValueArgs {
						identifier: "John Doe".into(),
						true_value: "Positive".into(),
					})?;
				"#
			).into(),
		});
		ui::Window::new()
			.child(
				ui::CodeSelect::new("prediction-threshold", code_for_language)
					.hide_line_numbers(false),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ProductionStats {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ProductionStats {
	fn into_node(self) -> Node {
		let rows = vec![
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "age".into(),
				column_type: ColumnType::Number,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "gender".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: Some("High Invalid Values Count".into()),
				name: "chest_pain".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "resting_blood_pressure".into(),
				column_type: ColumnType::Number,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "cholesterol".into(),
				column_type: ColumnType::Number,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "fasting_blood_sugar_greater_than_120".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "resting_ecg_result".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "exercise_max_heart_rate".into(),
				column_type: ColumnType::Number,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "exercise_induced_angina".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "exercise_st_depression".into(),
				column_type: ColumnType::Number,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "exercise_st_slope".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "fluoroscopy_vessels_colored".into(),
				column_type: ColumnType::Enum,
			},
			ColumnStatsTableRow {
				absent_count: 0,
				invalid_count: 0,
				href: None,
				alert: None,
				name: "thallium_stress_test".into(),
				column_type: ColumnType::Enum,
			},
		];
		ui::Window::new()
			.child(
				ui::S1::new()
					.child(ui::H1::new().child("Production Stats"))
					.child(ui::H2::new().child("Column Stats"))
					.child(ColumnStatsTable::new(rows)),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ProductionMetrics {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ProductionMetrics {
	fn into_node(self) -> Node {
		ui::Window::new()
			.child(
				ui::S1::new()
					.child(ui::H1::new().child("Production Metrics"))
					.child(
						ui::NumberComparisonCard::new(Some(0.83333), Some(0.78867))
							.color_a(ui::colors::BLUE.to_owned())
							.color_b(ui::colors::GREEN.to_owned())
							.title("Accuracy".to_owned())
							.value_a_title("Training".to_owned())
							.value_b_title("Production".to_owned())
							.number_formatter(ui::NumberFormatter::Percent(Default::default())),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ProductionColumnStats {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ProductionColumnStats {
	fn into_node(self) -> Node {
		let chart_data = vec![
			(
				"asymptomatic".to_owned(),
				EnumColumnOverallHistogramEntry {
					production_count: 95,
					production_fraction: Some(0.2311),
					training_count: 133,
					training_fraction: 0.4872,
				},
			),
			(
				"atypical angina".to_owned(),
				EnumColumnOverallHistogramEntry {
					production_count: 76,
					production_fraction: Some(0.1849),
					training_count: 43,
					training_fraction: 0.1575,
				},
			),
			(
				"non-angina pain".to_owned(),
				EnumColumnOverallHistogramEntry {
					training_count: 76,
					production_count: 122,
					training_fraction: 0.2784,
					production_fraction: Some(0.2968),
				},
			),
			(
				"typical angina".to_owned(),
				EnumColumnOverallHistogramEntry {
					training_count: 21,
					production_count: 25,
					training_fraction: 0.0769,
					production_fraction: Some(0.0608),
				},
			),
		];

		let enum_unique_values_table_props = EnumUniqueValuesTable {
			rows: vec![
				EnumUniqueValuesTableRow {
					name: "asymptomatic".to_owned(),
					production_count: 95,
					production_fraction: Some(0.2311),
					training_count: 133,
					training_fraction: 0.4872,
				},
				EnumUniqueValuesTableRow {
					name: "atypical angina".to_owned(),
					production_count: 76,
					production_fraction: Some(0.1849),
					training_count: 43,
					training_fraction: 0.1575,
				},
				EnumUniqueValuesTableRow {
					name: "non-angina pain".to_owned(),
					training_count: 76,
					production_count: 122,
					training_fraction: 0.2784,
					production_fraction: Some(0.2968),
				},
				EnumUniqueValuesTableRow {
					name: "typical angina".to_owned(),
					training_count: 21,
					production_count: 25,
					training_fraction: 0.0769,
					production_fraction: Some(0.0608),
				},
			],
		};
		let enum_invalid_values_table_props = EnumInvalidValuesTable {
			rows: vec![EnumInvalidValuesTableRow {
				name: "asx".to_owned(),
				count: 93,
				production_fraction: 0.2263,
			}],
		};
		ui::Window::new()
			.child(
				ui::S1::new()
					.child(ui::Alert::new(ui::Level::Danger).child("High Invalid Values Count"))
					.child(ui::H1::new().child("chest_pain"))
					.child(EnumColumnStatsSection::new(
						chart_data,
						"chest_pain".to_owned(),
						DateWindow::ThisMonth,
					))
					.child(EnumColumnUniqueValuesSection::new(
						enum_unique_values_table_props,
					))
					.child(EnumColumnInvalidValuesSection::new(
						enum_invalid_values_table_props,
					)),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ProductionExplanations {
	#[children]
	pub children: Vec<Node>,
}

impl Component for ProductionExplanations {
	fn into_node(self) -> Node {
		let class_name = "Positive".to_owned();
		let probability = 0.9748272;
		let feature_contributions_chart_data = vec![FeatureContributionsChartSeries {
			title: "".to_owned(),
			baseline: 0.02783647,
			baseline_label: "51%".to_owned(),
			output: 3.6564934,
			output_label: "97%".to_owned(),
			values: vec![
				FeatureContributionsChartValue {
					feature: "thallium_stress_test = 'normal'".to_owned(),
					value: -0.39572704,
				},
				FeatureContributionsChartValue {
					feature: "resting_blood_pressure = '160'".to_owned(),
					value: -0.14125186,
				},
				FeatureContributionsChartValue {
					feature: "gender = 'male'".to_owned(),
					value: 0.0,
				},
				FeatureContributionsChartValue {
					feature: "fasting_blood_sugar_greater_than_120 = 'false'".to_owned(),
					value: 0.0,
				},
				FeatureContributionsChartValue {
					feature:
						"resting_ecg_result = 'probable or definite left ventricular hypertrophy'"
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
					value: 0.13252445,
				},
				FeatureContributionsChartValue {
					feature: "exercise_st_slope = 'flat'".to_owned(),
					value: 0.17108941,
				},
				FeatureContributionsChartValue {
					feature: "exercise_st_depression = '1.5'".to_owned(),
					value: 0.22638911,
				},
				FeatureContributionsChartValue {
					feature: "chest_pain = 'asymptomatic'".to_owned(),
					value: 0.7210883,
				},
				FeatureContributionsChartValue {
					feature: "exercise_max_heart_rate = '108'".to_owned(),
					value: 1.1283911,
				},
				FeatureContributionsChartValue {
					feature: "cholesterol = '286'".to_owned(),
					value: 1.7861533,
				},
			],
			..Default::default()
		}];
		let feature_contributions_chart = FeatureContributionsChart::new(
			ui::colors::RED.to_owned(),
			ui::colors::GREEN.to_owned(),
			feature_contributions_chart_data,
		)
		.id("production-explanations".to_owned());
		ui::Window::new()
			.child(
				ui::S1::new()
					.child(ui::H1::new().child("Production Predictions".to_owned()))
					.child(
						div()
							.class("production-explanations-grid")
							.child(
								div()
									.style(style::GRID_AREA, "prediction")
									.child(ui::NumberCard::new("Predicted Class", class_name)),
							)
							.child(div().style(style::GRID_AREA, "probability").child(
								ui::NumberCard::new("Probability", ui::format_percent(probability)),
							))
							.child(
								div()
									.style(style::GRID_AREA, "feature-contributions")
									.child(feature_contributions_chart),
							),
					),
			)
			.into_node()
	}
}
