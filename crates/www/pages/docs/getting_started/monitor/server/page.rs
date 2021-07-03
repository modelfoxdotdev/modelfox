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

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let p1 = ui::P::new()
			.child("Once our model is deployed, we want to make sure that it performs as well in production as it did in training. We can opt in to logging by calling ")
			.child(ui::InlineCode::new("logPrediction"))
			.child(". Later on, as we get official diagnoses for patients, we can call ")
			.child(ui::InlineCode::new("logTrueValue"))
			.child(" and use the same identifier as we used in the call to ")
			.child(ui::InlineCode::new("logPrediction"))
			.child(".");
		let p2 = ui::P::new().child("Back in the app, we can look up a prediction by its identifier, and get an explanation that shows how each feature affects the output.");
		let p3 = ui::P::new().child("Now let's see how accurate our model has been in production. Let's open the app and choose Production Metrics in the sidebar.");
		let p4 = ui::P::new().child(r#"Uh oh! It's a bit lower than we expected. Let's try to find the cause. Under "Production Stats", we see that the "chest_pain" column has an alert and a high invalid values count. Click on the column to view more details."#);
		let p5 = ui::P::new().child(r#"It looks like there is a large discrepancy between the value "asymptomatic" in production versus training. In the table below, we see a high number of invalid values with the string "asx". It looks like we are accidentally using the string "asx" in our code instead of "asymptomatic" for the chest pain column. We can update our code to use the correct value and follow the metrics going forward to confirm they bounce back."#);
		let p6 = ui::Markdown::new("Hooray! You made it to the end! In this guide, we learned how to train a model, make predictions from our code, tune our model, and monitor it in production. If you want help using Tangram with your own data, send us an email at [hello@tangram.xyz](mailto:hello@tangram.xyz) or ask a question on [GitHub Discussions](https://github.com/tangramxyz/tangram/discussions).".into());
		let section = ui::S2::new()
			.child(p1)
			.child(Log)
			.child(p2)
			.child(ProductionExplanations)
			.child(p3)
			.child(ProductionMetrics)
			.child(p4)
			.child(ProductionStats)
			.child(p5)
			.child(ProductionColumnStats)
			.child(p6);
		let buttons = div().class("docs-prev-next-buttons").child(
			ui::Link::new()
				.href("inspect".to_owned())
				.child("< Previous: Inspect your model."),
		);
		Document::new()
			.client("tangram_www_docs_getting_started_monitor_client")
			.child(
				DocsLayout::new()
					.selected_page(DocsPage::GettingStarted(GettingStartedPage::Monitor))
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

pub struct Log;

impl Component for Log {
	fn into_node(self) -> Node {
		let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
			elixir: ui::doc!(
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
			go: ui::doc!(
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
			javascript: ui::doc!(
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
			python: ui::doc!(
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
			ruby: ui::doc!(
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
			rust: ui::doc!(
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
			.child(ui::CodeSelect::new(code_for_language).line_numbers(true))
			.into_node()
	}
}

pub struct ProductionStats;

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
					.child(ColumnStatsTable { rows }),
			)
			.into_node()
	}
}

pub struct ProductionMetrics;

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

pub struct ProductionColumnStats;

impl Component for ProductionColumnStats {
	fn into_node(self) -> Node {
		let overall_chart_data = vec![
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
					.child(EnumColumnStatsSection {
						overall_chart_data,
						column_name: "chest_pain".to_owned(),
						date_window: DateWindow::ThisMonth,
					})
					.child(EnumColumnUniqueValuesSection {
						enum_unique_values_table: enum_unique_values_table_props,
					})
					.child(EnumColumnInvalidValuesSection {
						enum_invalid_values_table: Some(enum_invalid_values_table_props),
					}),
			)
			.into_node()
	}
}

pub struct ProductionExplanations;

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
		let feature_contributions_chart = Dehydrate::new(
			"production-explanations",
			FeatureContributionsChart::new()
				.series(feature_contributions_chart_data)
				.negative_color(ui::colors::RED.to_owned())
				.positive_color(ui::colors::GREEN.to_owned()),
		);
		ui::Window::new()
			.child(
				ui::S1::new()
					.child(ui::H1::new().child("Production Predictions".to_owned()))
					.child(
						div()
							.class("production-explanations-grid")
							.child(div().style(style::GRID_AREA, "prediction").child(
								ui::NumberCard::new("Predicted Class".to_owned(), class_name),
							))
							.child(div().style(style::GRID_AREA, "probability").child(
								ui::NumberCard::new(
									"Probability".to_owned(),
									ui::format_percent(probability),
								),
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
