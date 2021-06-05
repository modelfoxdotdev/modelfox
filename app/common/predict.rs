use num::ToPrimitive;
use pinwheel::prelude::*;
use tangram_app_ui::{
	metrics_row::MetricsRow,
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken, UnknownColumnToken},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::{BarChart, FeatureContributionsChart},
	feature_contributions_chart::{
		FeatureContributionsChartSeries, FeatureContributionsChartValue,
	},
};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct PredictOutput {
	pub inner: PredictOutputInner,
	pub input_table: InputTable,
}

pub enum PredictOutputInner {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

impl Component for PredictOutput {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			PredictOutputInner::Regression(inner) => inner.into_node(),
			PredictOutputInner::BinaryClassification(inner) => inner.into_node(),
			PredictOutputInner::MulticlassClassification(inner) => inner.into_node(),
		};
		fragment()
			.child(
				ui::S2::new()
					.child(ui::H2::new().child("Input"))
					.child(self.input_table),
			)
			.child(inner)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct InputTable {
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

impl Component for InputTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Column Name"))
						.child(ui::TableHeaderCell::new().child("Column Type"))
						.child(ui::TableHeaderCell::new().child("Value")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					ui::TableRow::new()
						.child(ui::TableCell::new().child(row.column_name.to_owned()))
						.child(ui::TableCell::new().child(ColumnTypeToken::new(row.column_type)))
						.child(ui::TableCell::new().child(row.value))
				})),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ColumnTypeToken {
	column_type: InputTableColumnType,
}

impl Component for ColumnTypeToken {
	fn into_node(self) -> Node {
		match self.column_type {
			InputTableColumnType::Unknown => UnknownColumnToken::new().into(),
			InputTableColumnType::Number => NumberColumnToken::new().into(),
			InputTableColumnType::Enum => EnumColumnToken::new().into(),
			InputTableColumnType::Text => TextColumnToken::new().into(),
		}
	}
}

pub fn compute_input_table(
	model: tangram_model::ModelReader,
	input: &tangram_core::predict::PredictInput,
) -> InputTable {
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
		.collect::<Vec<_>>();
	InputTable::new(rows)
}

#[derive(ComponentBuilder)]
pub struct RegressionPredictOutput {
	pub feature_contributions_chart_series: FeatureContributionsChartSeries,
	pub value: f32,
}

impl Component for RegressionPredictOutput {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new().child("Output"))
			.child(ui::NumberCard::new(
				"Prediction".to_owned(),
				ui::format_float(self.value),
			))
			.child(ui::H2::new().child("Explanation"))
			.child(
				ui::P::new()
					.child("This chart shows how the features contributed to the model's output."),
			)
			.child(
				ui::Card::new().child(
					FeatureContributionsChart::new(
						ui::colors::RED.to_owned(),
						ui::colors::GREEN.to_owned(),
						vec![self.feature_contributions_chart_series],
					)
					.id("regression_feature_contributions".to_owned())
					.include_x_axis_title(Some(true))
					.include_y_axis_labels(Some(false))
					.include_y_axis_title(Some(false)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct BinaryClassificationPredictOutput {
	pub class_name: String,
	pub feature_contributions_chart_series: FeatureContributionsChartSeries,
	pub probability: f32,
}

impl Component for BinaryClassificationPredictOutput {
	fn into_node(self) -> Node {
		let output = ui::S2::new().child(ui::H2::new().child("Output")).child(
			MetricsRow::new()
				.child(ui::NumberCard::new(
					"Prediction".to_owned(),
					self.class_name,
				))
				.child(ui::NumberCard::new(
					"Probability".to_owned(),
					ui::format_percent(self.probability),
				)),
		);
		let explanation = ui::S2::new()
			.child(ui::H2::new().child("Explanation"))
			.child(
				ui::P::new()
					.child("This chart shows how the features contributed to the model's output."),
			)
			.child(
				ui::Card::new().child(
					FeatureContributionsChart::new(
						ui::colors::RED.to_owned(),
						ui::colors::GREEN.to_owned(),
						vec![self.feature_contributions_chart_series],
					)
					.id("binary_classification_feature_contributions".to_owned())
					.include_x_axis_title(Some(true))
					.include_y_axis_labels(Some(true))
					.include_y_axis_title(Some(true)),
				),
			);
		fragment().child(output).child(explanation).into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct MulticlassClassificationPredictOutput {
	pub class_name: String,
	pub feature_contributions_chart_series: Vec<FeatureContributionsChartSeries>,
	pub probabilities: Vec<(String, f32)>,
	pub probability: f32,
}

impl Component for MulticlassClassificationPredictOutput {
	fn into_node(self) -> Node {
		let probability_series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_owned(),
			title: Some("Probabilities".to_owned()),
			data: self
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
		let series = vec![self
			.feature_contributions_chart_series
			.iter()
			.find(|series| series.title == self.class_name)
			.unwrap()
			.clone()];
		ui::S2::new()
			.child(ui::H2::new().child("Output"))
			.child(ui::NumberCard::new(
				"Prediction".to_owned(),
				self.class_name,
			))
			.child(ui::NumberCard::new(
				"Probability".to_owned(),
				ui::format_percent(self.probability),
			))
			.child(
				BarChart::new()
					.id("probabilities".to_owned())
					.series(Some(probability_series))
					.title("Predicted Probabilities".to_owned())
					.y_min(Some(0.0)),
			)
			.child(ui::H2::new().child("Explanation"))
			.child(
				ui::P::new()
					.child("This chart shows how the features contributed to the model's output."),
			)
			.child(
				FeatureContributionsChart::new(
					ui::colors::RED.to_owned(),
					ui::colors::GREEN.to_owned(),
					series,
				)
				.id("multiclass_classification_feature_contributions".to_owned())
				.include_x_axis_title(Some(true))
				.include_y_axis_labels(Some(false))
				.include_y_axis_title(Some(true)),
			)
			.into_node()
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
			let predicate = if feature_contribution.feature_value != 0.0 {
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
		tangram_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(
			feature_contribution,
		) => {
			let feature = format!(
				"similarity of {} and {}",
				feature_contribution.column_name_a, feature_contribution.column_name_b,
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
