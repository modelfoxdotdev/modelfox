use crate::{
	metrics_row::MetricsRow,
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken},
};
use html::{component, html, Props};
use num::ToPrimitive;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::{BarChart, FeatureContributionsChart},
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PredictOutputProps {
	pub inner: PredictOutputInnerProps,
	pub input_table: InputTableProps,
}

pub enum PredictOutputInnerProps {
	Regression(RegressionPredictOutputProps),
	BinaryClassification(BinaryClassificationPredictOutputProps),
	MulticlassClassification(MulticlassClassificationPredictOutputProps),
}

#[component]
pub fn PredictOutput(props: PredictOutputProps) {
	let inner = match props.inner {
		PredictOutputInnerProps::Regression(inner) => {
			html! { <RegressionPredictOutput {inner} /> }
		}
		PredictOutputInnerProps::BinaryClassification(inner) => {
			html! { <BinaryClassificationPredictOutput {inner} /> }
		}
		PredictOutputInnerProps::MulticlassClassification(inner) => {
			html! { <MulticlassClassificationPredictOutput {inner} /> }
		}
	};
	html! {
		<>
			<ui::S2>
				<ui::H2>{"Input"}</ui::H2>
				<InputTable {props.input_table} />
			</ui::S2>
			{inner}
		</>
	}
}

#[derive(Props)]
pub struct InputTableProps {
	pub rows: Vec<InputTableRow>,
}

pub struct InputTableRow {
	pub column_name: String,
	pub column_type: InputTableColumnType,
	pub value: Option<String>,
}

#[derive(Clone, Copy)]
pub enum InputTableColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

#[component]
pub fn InputTable(props: InputTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Column Name"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Column Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Value"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.into_iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
							{row.column_name.to_owned()}
						</ui::TableCell>
						<ui::TableCell>
							<ColumnTypeToken column_type={row.column_type} />
						</ui::TableCell>
						<ui::TableCell>
							{row.value}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct ColumnTypeTokenProps {
	column_type: InputTableColumnType,
}

#[component]
fn ColumnTypeToken(props: ColumnTypeTokenProps) -> html::Node {
	match props.column_type {
		InputTableColumnType::Unknown => {
			html! { <UnknownColumnToken /> }
		}
		InputTableColumnType::Number => {
			html! { <NumberColumnToken /> }
		}
		InputTableColumnType::Enum => {
			html! { <EnumColumnToken /> }
		}
		InputTableColumnType::Text => {
			html! { <TextColumnToken /> }
		}
	}
}

pub fn compute_input_table_props(
	model: tangram_model::ModelReader,
	input: &tangram_core::predict::PredictInput,
) -> InputTableProps {
	let column_stats = match model.inner() {
		tangram_model::ModelInnerReader::Regressor(regressor) => {
			regressor.read().overall_column_stats()
		}
		tangram_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			binary_classifier.read().overall_column_stats()
		}
		tangram_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read().overall_column_stats()
		}
	};
	let rows = column_stats
		.iter()
		.map(|column_stats| match column_stats {
			tangram_model::ColumnStatsReader::UnknownColumn(column_stats) => {
				let column_stats = column_stats.read();
				let column_name = column_stats.column_name().to_owned();
				let value = input.0.get(&column_name).map(|value| match value {
					tangram_core::predict::PredictInputValue::Number(n) => n.to_string(),
					tangram_core::predict::PredictInputValue::String(s) => s.clone(),
				});
				InputTableRow {
					column_name,
					value,
					column_type: InputTableColumnType::Unknown,
				}
			}
			tangram_model::ColumnStatsReader::NumberColumn(column_stats) => {
				let column_stats = column_stats.read();
				let column_name = column_stats.column_name().to_owned();
				let value = input.0.get(&column_name).map(|value| match value {
					tangram_core::predict::PredictInputValue::Number(n) => n.to_string(),
					tangram_core::predict::PredictInputValue::String(s) => s.clone(),
				});
				InputTableRow {
					column_name,
					value,
					column_type: InputTableColumnType::Number,
				}
			}
			tangram_model::ColumnStatsReader::EnumColumn(column_stats) => {
				let column_stats = column_stats.read();
				let column_name = column_stats.column_name().to_owned();
				let value = input.0.get(&column_name).map(|value| match value {
					tangram_core::predict::PredictInputValue::Number(n) => n.to_string(),
					tangram_core::predict::PredictInputValue::String(s) => s.clone(),
				});
				InputTableRow {
					column_name,
					value,
					column_type: InputTableColumnType::Enum,
				}
			}
			tangram_model::ColumnStatsReader::TextColumn(column_stats) => {
				let column_stats = column_stats.read();
				let column_name = column_stats.column_name().to_owned();
				let value = input.0.get(&column_name).map(|value| match value {
					tangram_core::predict::PredictInputValue::Number(n) => n.to_string(),
					tangram_core::predict::PredictInputValue::String(s) => s.clone(),
				});
				InputTableRow {
					column_name,
					value,
					column_type: InputTableColumnType::Text,
				}
			}
		})
		.collect();
	InputTableProps { rows }
}

#[derive(Props)]
pub struct RegressionPredictOutputProps {
	pub feature_contributions_chart_series: FeatureContributionsChartSeries,
	pub value: f32,
}

#[component]
pub fn RegressionPredictOutput(props: RegressionPredictOutputProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Output"}</ui::H2>
			<ui::NumberCard
				title="Prediction"
				value={ui::format_float(props.value)}
			/>
			<ui::H2>{"Explanation"}</ui::H2>
			<ui::P>
				{"This chart shows how the features contributed to the model's output."}
			</ui::P>
			<ui::Card>
				<FeatureContributionsChart
					id?="regression_feature_contributions"
					include_x_axis_title?={Some(true)}
					include_y_axis_labels?={Some(false)}
					include_y_axis_title?={Some(false)}
					negative_color={ui::colors::RED.to_owned()}
					positive_color={ui::colors::GREEN.to_owned()}
					series={vec![props.feature_contributions_chart_series]}
				/>
			</ui::Card>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct BinaryClassificationPredictOutputProps {
	pub class_name: String,
	pub feature_contributions_chart_series: FeatureContributionsChartSeries,
	pub probability: f32,
}

#[component]
pub fn BinaryClassificationPredictOutput(props: BinaryClassificationPredictOutputProps) {
	html! {
		<>
			<ui::S2>
				<ui::H2>{"Output"}</ui::H2>
				<MetricsRow>
					<ui::NumberCard
						title="Prediction"
						value={props.class_name}
					/>
					<ui::NumberCard
						title="Probability"
						value={ui::format_percent(props.probability)}
					/>
				</MetricsRow>
			</ui::S2>
			<ui::S2>
				<ui::H2>{"Explanation"}</ui::H2>
				<ui::P>
					{"This chart shows how the features contributed to the model's output."}
				</ui::P>
				<ui::Card>
					<FeatureContributionsChart
						id?="binary_classification_feature_contributions"
						include_x_axis_title?={Some(true)}
						include_y_axis_labels?={Some(true)}
						include_y_axis_title?={Some(true)}
						negative_color={ui::colors::RED.to_owned()}
						positive_color={ui::colors::GREEN.to_owned()}
						series={vec![props.feature_contributions_chart_series]}
					/>
				</ui::Card>
			</ui::S2>
		</>
	}
}

#[derive(Props)]
pub struct MulticlassClassificationPredictOutputProps {
	pub class_name: String,
	pub feature_contributions_chart_series: Vec<FeatureContributionsChartSeries>,
	pub probabilities: Vec<(String, f32)>,
	pub probability: f32,
}

#[component]
pub fn MulticlassClassificationPredictOutput(props: MulticlassClassificationPredictOutputProps) {
	let probability_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		title: Some("Probabilities".to_owned()),
		data: props
			.probabilities
			.iter()
			.enumerate()
			.map(|(index, (class_name, probability))| BarChartPoint {
				label: class_name.to_owned(),
				x: index.to_f64().unwrap(),
				y: Some(probability.to_f64().unwrap()),
			})
			.collect::<Vec<_>>(),
	}];
	let series = vec![props
		.feature_contributions_chart_series
		.iter()
		.find(|series| series.title == props.class_name)
		.unwrap()
		.clone()];
	html! {
		<ui::S2>
			<ui::H2>{"Output"}</ui::H2>
			<ui::NumberCard
				title="Prediction"
				value={props.class_name}
			/>
			<ui::NumberCard
				title="Probability"
				value={ui::format_percent(props.probability)}
			/>
			<BarChart
				id?="probabilities"
				series?={Some(probability_series)}
				title?="Predicted Probabilities"
				y_min?={Some(0.0)}
			/>
			<ui::H2>{"Explanation"}</ui::H2>
			<ui::P>
				{"This chart shows how the features contributed to the model's output."}
			</ui::P>
			<FeatureContributionsChart
				id?="multiclass_classification_feature_contributions"
				include_x_axis_title?={Some(true)}
				include_y_axis_labels?={Some(false)}
				include_y_axis_title?={Some(true)}
				negative_color={ui::colors::RED.to_owned()}
				positive_color={ui::colors::GREEN.to_owned()}
				series={series}
			/>
		</ui::S2>
	}
}

pub fn compute_feature_contributions_chart_series(
	title: String,
	feature_contributions: tangram_core::predict::FeatureContributions,
) -> FeatureContributionsChartSeries {
	FeatureContributionsChartSeries {
		baseline: feature_contributions.baseline_value.to_f64().unwrap(),
		baseline_label: ui::format_float_with_digits(feature_contributions.baseline_value, 3),
		output: feature_contributions.output_value.to_f64().unwrap(),
		output_label: ui::format_float_with_digits(feature_contributions.output_value, 3),
		title,
		values: feature_contributions
			.entries
			.into_iter()
			.map(compute_feature_contributions_chart_value)
			.collect(),
		..Default::default()
	}
}

fn compute_feature_contributions_chart_value(
	entry: tangram_core::predict::FeatureContributionEntry,
) -> FeatureContributionsChartValue {
	match entry {
		tangram_core::predict::FeatureContributionEntry::Identity(feature_contribution) => {
			FeatureContributionsChartValue {
				feature: feature_contribution.column_name,
				value: feature_contribution
					.feature_contribution_value
					.to_f64()
					.unwrap(),
			}
		}
		tangram_core::predict::FeatureContributionEntry::Normalized(feature_contribution) => {
			FeatureContributionsChartValue {
				feature: feature_contribution.column_name,
				value: feature_contribution
					.feature_contribution_value
					.to_f64()
					.unwrap(),
			}
		}
		tangram_core::predict::FeatureContributionEntry::OneHotEncoded(feature_contribution) => {
			let predicate = if feature_contribution.feature_value {
				"is"
			} else {
				"is not"
			};
			let variant = feature_contribution
				.variant
				.map(|variant| format!("\"{}\"", variant))
				.unwrap_or_else(|| "invalid".to_owned());
			let feature = format!(
				"{} {} {}",
				feature_contribution.column_name, predicate, variant
			);
			FeatureContributionsChartValue {
				feature,
				value: feature_contribution
					.feature_contribution_value
					.to_f64()
					.unwrap(),
			}
		}
		tangram_core::predict::FeatureContributionEntry::BagOfWords(feature_contribution) => {
			let predicate = if feature_contribution.feature_value {
				"contains"
			} else {
				"does not contain"
			};
			let feature = format!(
				"{} {} \"{}\"",
				feature_contribution.column_name, predicate, feature_contribution.ngram
			);
			FeatureContributionsChartValue {
				feature,
				value: feature_contribution
					.feature_contribution_value
					.to_f64()
					.unwrap(),
			}
		}
		tangram_core::predict::FeatureContributionEntry::WordEmbedding(feature_contribution) => {
			let feature = format!("{} word model", feature_contribution.column_name);
			FeatureContributionsChartValue {
				feature,
				value: feature_contribution
					.feature_contribution_value
					.to_f64()
					.unwrap(),
			}
		}
	}
}
