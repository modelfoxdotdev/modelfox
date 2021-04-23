use html::{component, html, style};
use indoc::indoc;
use tangram_app_common::{column_type::ColumnType, date_window::DateWindow};
use tangram_app_production_stats_column_server::{
	EnumColumnOverallHistogramEntry, EnumInvalidValuesSection, EnumInvalidValuesTableProps,
	EnumInvalidValuesTableRow, EnumStatsSection, EnumUniqueValuesSection,
	EnumUniqueValuesTableProps, EnumUniqueValuesTableRow,
};
use tangram_app_production_stats_index_server::{ColumnStatsTable, ColumnStatsTableRow};
use tangram_charts::{
	components::FeatureContributionsChart,
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_serve::client;
use tangram_ui as ui;
use tangram_www_layouts::docs_app_layout::DocsAppLayout;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Monitor)} headings={None}>
				<ui::S1>
					<ui::H1>{"Monitor"}</ui::H1>
					<ui::S2>
						<ui::P>
							{"Once our model is deployed, we want to make sure that it performs as well in production as it did in training. We can opt in to logging predictions by calling "}<ui::InlineCode>{"logPrediction"}</ui::InlineCode>
							{". Later on, as we get official diagnoses for patients, we can call "}<ui::InlineCode> {"logTrueValue"}</ui::InlineCode>{" and use the same identifier as we used in the call to "}<ui::InlineCode>{"logPrediction"}</ui::InlineCode>
							{"."}
						</ui::P>
						<Log/>
						<ui::P>
							{"Back in the app, we can look up a prediction by its identifier, and get an explanation which shows how each feature affects the output."}
						</ui::P>
						<ProductionExplanations />
						<ui::P>
							{"Now let's see how accurate our model has been in production. Lets open the app and view 'Production Metrics'"}
						</ui::P>
						<ui::P>
						</ui::P>
							<ProductionMetrics />
						<ui::P>
							{"Uh oh! It's a bit lower than we expected. Let's try to find the cause. Under 'Production Stats', we see that the 'chest_pain' column has an alert and a high unknown values count. Click on the column to view more details."}
						</ui::P>
						<ProductionStats />
						<ui::P>
							{"It looks like there is a large discrepancy between the value 'asymptomatic' in production versus training. In the table below, we see a high number of invalid values with the string 'asx'. It looks like we are accidentally using the string 'asx' in our code instead of 'asymptomatic' for the chest pain column. We can update our code to use the correct value and follow the metrics going forward to confirm they bounce back."}
						</ui::P>
						<ProductionColumnStats />
					</ui::S2>
					<div class="docs-prev-next-buttons">
						<ui::Link href="inspect">
							{"< Previous: Inspect your model."}
						</ui::Link>
					</div>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}

#[component]
pub fn Log() {
	let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
		elixir: indoc! {
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
		}.into(),
		go: indoc! {
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
		}.into(),
		javascript: indoc!{
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
		}.into(),
		python: indoc! {
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
		}.into(),
		ruby: indoc! {
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
		}.into(),
		rust: indoc! {
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
		}
		.into(),
	});
	html! {
		<ui::Window padding={Some(true)}>
			<ui::CodeSelect
				id="predict-threshold"
				code_for_language={code_for_language}
				hide_line_numbers?={Some(false)}
			/>
		</ui::Window>
	}
}

#[component]
pub fn ProductionStats() {
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
	html! {
		<DocsAppLayout>
			<ui::H1>{"Production Stats"}</ui::H1>
			<ui::H2>{"Column Stats"}</ui::H2>
			<ColumnStatsTable rows={rows} />
		</DocsAppLayout>
	}
}

#[component]
pub fn ProductionMetrics() {
	html! {
		<DocsAppLayout>
			<ui::H1>{"Production Metrics"}</ui::H1>
			<ui::NumberComparisonCard
				color_a={Some(ui::colors::BLUE.to_owned())}
				color_b={Some(ui::colors::GREEN.to_owned())}
				title="Accuracy"
				value_a={Some(0.8333)}
				value_a_title="Training"
				value_b={Some(0.7849)}
				value_b_title="Production"
				number_formatter={ui::NumberFormatter::Percent(Default::default())}
			/>
		</DocsAppLayout>
	}
}

#[component]
pub fn ProductionColumnStats() {
	let chart_data = vec![
		(
			"typical angina".to_owned(),
			EnumColumnOverallHistogramEntry {
				production_count: 0,
				production_fraction: None,
				training_count: 133,
				training_fraction: 0.4872,
			},
		),
		(
			"atypical angina".to_owned(),
			EnumColumnOverallHistogramEntry {
				production_count: 0,
				production_fraction: None,
				training_count: 43,
				training_fraction: 0.1575,
			},
		),
		(
			"non-angina pain".to_owned(),
			EnumColumnOverallHistogramEntry {
				training_count: 76,
				production_count: 0,
				training_fraction: 0.2784,
				production_fraction: None,
			},
		),
		(
			"asymptomatic".to_owned(),
			EnumColumnOverallHistogramEntry {
				training_count: 21,
				production_count: 0,
				training_fraction: 0.0769,
				production_fraction: None,
			},
		),
	];

	let enum_unique_values_table_props = EnumUniqueValuesTableProps {
		rows: vec![
			EnumUniqueValuesTableRow {
				name: "typical angina".to_owned(),
				training_count: 133,
				production_count: 0,
				training_fraction: 0.4872,
				production_fraction: None,
			},
			EnumUniqueValuesTableRow {
				name: "atypical angina".to_owned(),
				training_count: 43,
				production_count: 0,
				training_fraction: 0.1575,
				production_fraction: None,
			},
			EnumUniqueValuesTableRow {
				name: "non-angina pain".to_owned(),
				training_count: 76,
				production_count: 0,
				training_fraction: 0.2784,
				production_fraction: None,
			},
			EnumUniqueValuesTableRow {
				name: "asymptomatic".to_owned(),
				training_count: 21,
				production_count: 0,
				training_fraction: 0.0769,
				production_fraction: None,
			},
		],
	};
	let enum_invalid_values_table_props = EnumInvalidValuesTableProps {
		rows: vec![EnumInvalidValuesTableRow {
			name: "asx".to_owned(),
			count: 38,
		}],
	};
	html! {
		<DocsAppLayout>
			<ui::Alert level={ui::Level::Danger}>
				{"High Invalid Values Count"}
			</ui::Alert>
			<ui::H1>{"chest_pain"}</ui::H1>
			<EnumStatsSection
				overall_chart_data={chart_data}
				column_name="chest_pain"
				date_window={DateWindow::ThisMonth}
			/>
			<EnumUniqueValuesSection
				enum_unique_values_table_props={enum_unique_values_table_props}
			/>
			<EnumInvalidValuesSection
				enum_invalid_values_table_props={Some(enum_invalid_values_table_props)}
			/>
		</DocsAppLayout>
	}
}

#[component]
pub fn ProductionExplanations() {
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
	html! {
		<DocsAppLayout>
			<ui::H1>{"Production Predictions"}</ui::H1>
			<div class="production-explanations-grid">
				<div style={style! { "grid-area" => "prediction" }}>
					<ui::NumberCard
						title="Predicted Class"
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
							negative_color={ui::colors::RED.to_owned()}
							positive_color={ui::colors::GREEN.to_owned()}
							series={feature_contributions_chart_data}
						/>
					</ui::Card>
				</div>
			</div>
		</DocsAppLayout>
	}
}
