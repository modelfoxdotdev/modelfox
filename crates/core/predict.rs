use modelfox_features::{
	bag_of_words::BagOfWordsFeatureGroupNGramEntry, BagOfWordsCosineSimilarityFeatureGroup,
	BagOfWordsFeatureGroup, FeatureGroup, IdentityFeatureGroup, NormalizedFeatureGroup,
	OneHotEncodedFeatureGroup, WordEmbeddingFeatureGroup,
};
use modelfox_table::prelude::*;
use modelfox_text::NGramType;
use modelfox_zip::zip;
use ndarray::prelude::*;
use num::ToPrimitive;
use std::collections::BTreeMap;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PredictInput(pub BTreeMap<String, PredictInputValue>);

impl Default for PredictInput {
	fn default() -> Self {
		PredictInput::new()
	}
}

impl PredictInput {
	pub fn new() -> PredictInput {
		PredictInput(BTreeMap::new())
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PredictInputValue {
	Number(f64),
	String(String),
}

impl PredictInputValue {
	pub fn as_number(&self) -> Option<f64> {
		match self {
			PredictInputValue::Number(n) => Some(*n),
			_ => None,
		}
	}
	pub fn as_str(&self) -> Option<&str> {
		match self {
			PredictInputValue::String(s) => Some(s.as_str()),
			_ => None,
		}
	}
}

impl From<f64> for PredictInputValue {
	fn from(value: f64) -> Self {
		PredictInputValue::Number(value)
	}
}

impl From<f32> for PredictInputValue {
	fn from(value: f32) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<i32> for PredictInputValue {
	fn from(value: i32) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<u32> for PredictInputValue {
	fn from(value: u32) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<i16> for PredictInputValue {
	fn from(value: i16) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<u16> for PredictInputValue {
	fn from(value: u16) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<i8> for PredictInputValue {
	fn from(value: i8) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}
impl From<u8> for PredictInputValue {
	fn from(value: u8) -> Self {
		PredictInputValue::Number(f64::from(value))
	}
}

impl From<String> for PredictInputValue {
	fn from(value: String) -> Self {
		PredictInputValue::String(value)
	}
}

impl From<&str> for PredictInputValue {
	fn from(value: &str) -> Self {
		PredictInputValue::String(value.to_owned())
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PredictOptions {
	pub threshold: f32,
	pub compute_feature_contributions: bool,
}

impl Default for PredictOptions {
	fn default() -> PredictOptions {
		PredictOptions {
			threshold: 0.5,
			compute_feature_contributions: false,
		}
	}
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum PredictOutput {
	#[serde(rename = "regression")]
	Regression(RegressionPredictOutput),
	#[serde(rename = "binary_classification")]
	BinaryClassification(BinaryClassificationPredictOutput),
	#[serde(rename = "multiclass_classification")]
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

#[derive(Debug, serde::Serialize)]
pub struct RegressionPredictOutput {
	pub value: f32,
	pub feature_contributions: Option<FeatureContributions>,
}

#[derive(Debug, serde::Serialize)]
pub struct BinaryClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub feature_contributions: Option<FeatureContributions>,
}

#[derive(Debug, serde::Serialize)]
pub struct MulticlassClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub probabilities: BTreeMap<String, f32>,
	pub feature_contributions: Option<BTreeMap<String, FeatureContributions>>,
}

#[derive(Debug, serde::Serialize)]
pub struct FeatureContributions {
	/// The baseline value is the value output by the model for this class before taking into account the feature values.
	pub baseline_value: f32,
	/// The output value is the sum of the baseline value and the feature contribution values of all features.
	pub output_value: f32,
	/// These are the feature contribution entries for each feature.
	pub entries: Vec<FeatureContributionEntry>,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum FeatureContributionEntry {
	#[serde(rename = "identity")]
	Identity(IdentityFeatureContribution),
	#[serde(rename = "normalized")]
	Normalized(NormalizedFeatureContribution),
	#[serde(rename = "one_hot_encoded")]
	OneHotEncoded(OneHotEncodedFeatureContribution),
	#[serde(rename = "bag_of_words")]
	BagOfWords(BagOfWordsFeatureContribution),
	#[serde(rename = "bag_of_words_cosine_similarity")]
	BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureContribution),
	#[serde(rename = "word_embedding")]
	WordEmbedding(WordEmbeddingFeatureContribution),
}

#[derive(Debug, serde::Serialize)]
pub struct IdentityFeatureContribution {
	pub column_name: String,
	pub feature_value: f32,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct NormalizedFeatureContribution {
	pub column_name: String,
	pub feature_value: f32,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct OneHotEncodedFeatureContribution {
	pub column_name: String,
	pub variant: Option<String>,
	pub feature_value: bool,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct BagOfWordsFeatureContribution {
	pub column_name: String,
	pub ngram: NGram,
	pub feature_value: f32,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct BagOfWordsCosineSimilarityFeatureContribution {
	pub column_name_a: String,
	pub column_name_b: String,
	pub feature_value: f32,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct WordEmbeddingFeatureContribution {
	pub column_name: String,
	pub value_index: usize,
	pub feature_contribution_value: f32,
}

#[derive(Debug, serde::Serialize)]
pub enum NGram {
	#[serde(rename = "unigram")]
	Unigram(String),
	#[serde(rename = "bigram")]
	Bigram(String, String),
}

impl From<modelfox_text::NGram> for NGram {
	fn from(value: modelfox_text::NGram) -> NGram {
		match value {
			modelfox_text::NGram::Unigram(token) => NGram::Unigram(token),
			modelfox_text::NGram::Bigram(token_a, token_b) => NGram::Bigram(token_a, token_b),
		}
	}
}

impl std::fmt::Display for NGram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NGram::Unigram(token) => write!(f, "{}", token),
			NGram::Bigram(token_a, token_b) => write!(f, "{} {}", token_a, token_b),
		}
	}
}

#[derive(Debug)]
pub struct Model {
	pub id: String,
	pub inner: ModelInner,
}

#[derive(Debug)]
pub enum ModelInner {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

#[derive(Debug)]
pub struct Regressor {
	pub columns: Vec<Column>,
	pub feature_groups: Vec<modelfox_features::FeatureGroup>,
	pub model: RegressionModel,
}

#[derive(Debug)]
pub struct BinaryClassifier {
	pub columns: Vec<Column>,
	pub negative_class: String,
	pub positive_class: String,
	pub feature_groups: Vec<modelfox_features::FeatureGroup>,
	pub model: BinaryClassificationModel,
}

#[derive(Debug)]
pub struct MulticlassClassifier {
	pub columns: Vec<Column>,
	pub classes: Vec<String>,
	pub feature_groups: Vec<modelfox_features::FeatureGroup>,
	pub model: MulticlassClassificationModel,
}

#[derive(Debug)]
pub enum RegressionModel {
	Linear(modelfox_linear::Regressor),
	Tree(modelfox_tree::Regressor),
}

#[derive(Debug)]
pub enum BinaryClassificationModel {
	Linear(modelfox_linear::BinaryClassifier),
	Tree(modelfox_tree::BinaryClassifier),
}

#[derive(Debug)]
pub enum MulticlassClassificationModel {
	Linear(modelfox_linear::MulticlassClassifier),
	Tree(modelfox_tree::MulticlassClassifier),
}

#[derive(Debug)]
pub enum Column {
	Unknown(UnknownColumn),
	Number(NumberColumn),
	Enum(EnumColumn),
	Text(TextColumn),
}

#[derive(Debug)]
pub struct UnknownColumn {
	name: String,
}

#[derive(Debug)]
pub struct NumberColumn {
	name: String,
}

#[derive(Debug)]
pub struct EnumColumn {
	name: String,
	variants: Vec<String>,
}

#[derive(Debug)]
pub struct TextColumn {
	name: String,
}

impl<'a> From<modelfox_model::ModelReader<'a>> for Model {
	fn from(model: modelfox_model::ModelReader<'a>) -> Self {
		deserialize_model(model)
	}
}

fn deserialize_model(model: modelfox_model::ModelReader) -> Model {
	let id = model.id().parse().unwrap();
	let inner = deserialize_model_inner(model.inner());
	Model { id, inner }
}

fn deserialize_model_inner(model_inner: modelfox_model::ModelInnerReader) -> ModelInner {
	match model_inner {
		modelfox_model::ModelInnerReader::Regressor(regressor) => {
			let regressor = regressor.read();
			let columns = regressor
				.train_column_stats()
				.iter()
				.map(deserialize_column_stats)
				.collect::<Vec<_>>();
			let feature_groups = match regressor.model() {
				modelfox_model::RegressionModelReader::Linear(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
				modelfox_model::RegressionModelReader::Tree(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
			};
			let model = match regressor.model() {
				modelfox_model::RegressionModelReader::Linear(model) => RegressionModel::Linear(
					modelfox_linear::Regressor::from_reader(model.read().model()),
				),
				modelfox_model::RegressionModelReader::Tree(model) => RegressionModel::Tree(
					modelfox_tree::Regressor::from_reader(model.read().model()),
				),
			};
			ModelInner::Regressor(Regressor {
				columns,
				feature_groups,
				model,
			})
		}
		modelfox_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			let binary_classifier = binary_classifier.read();
			let negative_class = binary_classifier.negative_class().to_owned();
			let positive_class = binary_classifier.positive_class().to_owned();
			let columns = binary_classifier
				.train_column_stats()
				.iter()
				.map(deserialize_column_stats)
				.collect::<Vec<_>>();
			let feature_groups = match binary_classifier.model() {
				modelfox_model::BinaryClassificationModelReader::Linear(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
				modelfox_model::BinaryClassificationModelReader::Tree(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
			};
			let model = match binary_classifier.model() {
				modelfox_model::BinaryClassificationModelReader::Linear(model) => {
					BinaryClassificationModel::Linear(
						modelfox_linear::BinaryClassifier::from_reader(model.read().model()),
					)
				}
				modelfox_model::BinaryClassificationModelReader::Tree(model) => {
					BinaryClassificationModel::Tree(modelfox_tree::BinaryClassifier::from_reader(
						model.read().model(),
					))
				}
			};
			ModelInner::BinaryClassifier(BinaryClassifier {
				columns,
				negative_class,
				positive_class,
				feature_groups,
				model,
			})
		}
		modelfox_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier = multiclass_classifier.read();
			let classes = multiclass_classifier
				.classes()
				.iter()
				.map(|class| class.to_owned())
				.collect::<Vec<_>>();
			let columns = multiclass_classifier
				.train_column_stats()
				.iter()
				.map(deserialize_column_stats)
				.collect::<Vec<_>>();
			let feature_groups = match multiclass_classifier.model() {
				modelfox_model::MulticlassClassificationModelReader::Linear(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
				modelfox_model::MulticlassClassificationModelReader::Tree(model) => model
					.read()
					.feature_groups()
					.iter()
					.map(deserialize_feature_group)
					.collect::<Vec<_>>(),
			};
			let model = match multiclass_classifier.model() {
				modelfox_model::MulticlassClassificationModelReader::Linear(model) => {
					MulticlassClassificationModel::Linear(
						modelfox_linear::MulticlassClassifier::from_reader(model.read().model()),
					)
				}
				modelfox_model::MulticlassClassificationModelReader::Tree(model) => {
					MulticlassClassificationModel::Tree(
						modelfox_tree::MulticlassClassifier::from_reader(model.read().model()),
					)
				}
			};
			ModelInner::MulticlassClassifier(MulticlassClassifier {
				columns,
				classes,
				feature_groups,
				model,
			})
		}
	}
}

fn deserialize_column_stats(column_stats: modelfox_model::ColumnStatsReader) -> Column {
	match column_stats {
		modelfox_model::ColumnStatsReader::UnknownColumn(column_stats) => {
			let column_stats = column_stats.read();
			let name = column_stats.column_name().to_owned();
			Column::Unknown(UnknownColumn { name })
		}
		modelfox_model::ColumnStatsReader::NumberColumn(column_stats) => {
			let column_stats = column_stats.read();
			let name = column_stats.column_name().to_owned();
			Column::Number(NumberColumn { name })
		}
		modelfox_model::ColumnStatsReader::EnumColumn(column_stats) => {
			let column_stats = column_stats.read();
			let name = column_stats.column_name().to_owned();
			let variants = column_stats
				.histogram()
				.iter()
				.map(|(key, _)| key.to_owned())
				.collect::<Vec<_>>();
			Column::Enum(EnumColumn { name, variants })
		}
		modelfox_model::ColumnStatsReader::TextColumn(column_stats) => {
			let column_stats = column_stats.read();
			let name = column_stats.column_name().to_owned();
			Column::Text(TextColumn { name })
		}
	}
}

fn deserialize_feature_group(feature_group: modelfox_model::FeatureGroupReader) -> FeatureGroup {
	match feature_group {
		modelfox_model::FeatureGroupReader::Identity(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name = feature_group.source_column_name().to_owned();
			FeatureGroup::Identity(IdentityFeatureGroup { source_column_name })
		}
		modelfox_model::FeatureGroupReader::Normalized(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name = feature_group.source_column_name().to_owned();
			let mean = feature_group.mean();
			let variance = feature_group.variance();
			FeatureGroup::Normalized(NormalizedFeatureGroup {
				source_column_name,
				mean,
				variance,
			})
		}
		modelfox_model::FeatureGroupReader::OneHotEncoded(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name = feature_group.source_column_name().to_owned();
			let variants = feature_group
				.variants()
				.iter()
				.map(|key| key.to_owned())
				.collect::<Vec<_>>();
			FeatureGroup::OneHotEncoded(OneHotEncodedFeatureGroup {
				source_column_name,
				variants,
			})
		}
		modelfox_model::FeatureGroupReader::BagOfWords(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name = feature_group.source_column_name().to_owned();
			let tokenizer = deserialize_tokenizer(feature_group.tokenizer());
			let strategy =
				deserialize_bag_of_words_feature_group_strategy(feature_group.strategy());
			let ngrams = feature_group
				.ngrams()
				.iter()
				.map(|(ngram, entry)| {
					(
						deserialize_ngram(ngram),
						BagOfWordsFeatureGroupNGramEntry { idf: entry.idf() },
					)
				})
				.collect();
			let ngram_types = feature_group
				.ngram_types()
				.iter()
				.map(|ngram_type| match ngram_type {
					modelfox_model::NGramTypeReader::Unigram(_) => NGramType::Unigram,
					modelfox_model::NGramTypeReader::Bigram(_) => NGramType::Bigram,
				})
				.collect();
			FeatureGroup::BagOfWords(BagOfWordsFeatureGroup {
				source_column_name,
				strategy,
				tokenizer,
				ngram_types,
				ngrams,
			})
		}
		modelfox_model::FeatureGroupReader::BagOfWordsCosineSimilarity(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name_a = feature_group.source_column_name_a().to_owned();
			let source_column_name_b = feature_group.source_column_name_b().to_owned();
			let tokenizer = deserialize_tokenizer(feature_group.tokenizer());
			let strategy =
				deserialize_bag_of_words_feature_group_strategy(feature_group.strategy());
			let ngrams = feature_group
				.ngrams()
				.iter()
				.map(|(ngram, entry)| {
					(
						deserialize_ngram(ngram),
						BagOfWordsFeatureGroupNGramEntry { idf: entry.idf() },
					)
				})
				.collect();
			let ngram_types = feature_group
				.ngram_types()
				.iter()
				.map(|ngram_type| match ngram_type {
					modelfox_model::NGramTypeReader::Unigram(_) => NGramType::Unigram,
					modelfox_model::NGramTypeReader::Bigram(_) => NGramType::Bigram,
				})
				.collect();
			FeatureGroup::BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureGroup {
				source_column_name_a,
				source_column_name_b,
				strategy,
				tokenizer,
				ngram_types,
				ngrams,
			})
		}
		modelfox_model::FeatureGroupReader::WordEmbedding(feature_group) => {
			let feature_group = feature_group.read();
			let source_column_name = feature_group.source_column_name().to_owned();
			let tokenizer = deserialize_tokenizer(feature_group.tokenizer());
			let model = deserialize_word_embedding_model(feature_group.model());
			FeatureGroup::WordEmbedding(WordEmbeddingFeatureGroup {
				source_column_name,
				tokenizer,
				model,
			})
		}
	}
}

fn deserialize_tokenizer(tokenizer: modelfox_model::TokenizerReader) -> modelfox_text::Tokenizer {
	tokenizer.into()
}

fn deserialize_bag_of_words_feature_group_strategy(
	strategy: modelfox_model::BagOfWordsFeatureGroupStrategyReader,
) -> modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy {
	match strategy {
		modelfox_model::BagOfWordsFeatureGroupStrategyReader::Present(_) => {
			modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Present
		}
		modelfox_model::BagOfWordsFeatureGroupStrategyReader::Count(_) => {
			modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Count
		}
		modelfox_model::BagOfWordsFeatureGroupStrategyReader::TfIdf(_) => {
			modelfox_features::bag_of_words::BagOfWordsFeatureGroupStrategy::TfIdf
		}
	}
}

fn deserialize_word_embedding_model(
	model: modelfox_model::WordEmbeddingModelReader,
) -> modelfox_text::WordEmbeddingModel {
	model.into()
}

fn deserialize_ngram(ngram: modelfox_model::NGramReader) -> modelfox_text::NGram {
	match ngram {
		modelfox_model::NGramReader::Unigram(unigram) => {
			let token = unigram.read();
			modelfox_text::NGram::Unigram((*token).to_owned())
		}
		modelfox_model::NGramReader::Bigram(bigram) => {
			let (token_a, token_b) = bigram.read();
			modelfox_text::NGram::Bigram(token_a.to_owned(), token_b.to_owned())
		}
	}
}

pub fn predict(
	model: &Model,
	input: &[PredictInput],
	options: &PredictOptions,
) -> Vec<PredictOutput> {
	// Initialize the table.
	let columns = match &model.inner {
		ModelInner::Regressor(regressor) => regressor.columns.as_slice(),
		ModelInner::BinaryClassifier(binary_classifier) => binary_classifier.columns.as_slice(),
		ModelInner::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.columns.as_slice()
		}
	};
	let column_names = columns
		.iter()
		.map(|column| match column {
			Column::Unknown(column) => Some(column.name.clone()),
			Column::Number(column) => Some(column.name.clone()),
			Column::Enum(column) => Some(column.name.clone()),
			Column::Text(column) => Some(column.name.clone()),
		})
		.collect();
	let column_types = columns
		.iter()
		.map(|column| match column {
			Column::Unknown(_) => modelfox_table::TableColumnType::Unknown,
			Column::Number(_) => modelfox_table::TableColumnType::Number,
			Column::Enum(column) => modelfox_table::TableColumnType::Enum {
				variants: column.variants.clone(),
			},
			Column::Text(_) => modelfox_table::TableColumnType::Text,
		})
		.collect();
	let mut table = modelfox_table::Table::new(column_names, column_types);
	// Fill the table with the input.
	for input in input {
		for column in table.columns_mut().iter_mut() {
			match column {
				modelfox_table::TableColumn::Unknown(column) => *column.len_mut() += 1,
				modelfox_table::TableColumn::Number(column) => {
					let value = match input.0.get(column.name().as_ref().unwrap()) {
						Some(PredictInputValue::Number(value)) => value.to_f32().unwrap(),
						Some(PredictInputValue::String(value)) => {
							fast_float::parse::<f32, &str>(value)
								.map(|value| if value.is_finite() { value } else { f32::NAN })
								.unwrap_or(f32::NAN)
						}
						_ => f32::NAN,
					};
					column.data_mut().push(value);
				}
				modelfox_table::TableColumn::Enum(column) => {
					let value = input
						.0
						.get(column.name().as_ref().unwrap())
						.and_then(|value| value.as_str());
					let value = value.and_then(|value| column.value_for_variant(value));
					column.data_mut().push(value);
				}
				modelfox_table::TableColumn::Text(column) => {
					let value = input
						.0
						.get(column.name().as_ref().unwrap())
						.and_then(|value| value.as_str())
						.unwrap_or("")
						.to_owned();
					column.data_mut().push(value);
				}
			}
		}
	}
	// Make the predictions by matching on the model type.
	match &model.inner {
		ModelInner::Regressor(regressor) => predict_regressor(regressor, table, options)
			.into_iter()
			.map(PredictOutput::Regression)
			.collect(),
		ModelInner::BinaryClassifier(model) => predict_binary_classifier(model, table, options)
			.into_iter()
			.map(PredictOutput::BinaryClassification)
			.collect(),
		ModelInner::MulticlassClassifier(model) => {
			predict_multiclass_classifier(model, table, options)
				.into_iter()
				.map(PredictOutput::MulticlassClassification)
				.collect()
		}
	}
}

fn predict_regressor(
	model: &Regressor,
	table: Table,
	options: &PredictOptions,
) -> Vec<RegressionPredictOutput> {
	let n_rows = table.nrows();
	match &model.model {
		RegressionModel::Linear(inner_model) => {
			let mut predictions = Array::zeros(n_rows);
			let features = modelfox_features::compute_features_array_f32(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), predictions.view_mut());
			let mut outputs: Vec<RegressionPredictOutput> = predictions
				.iter()
				.map(|prediction| RegressionPredictOutput {
					value: *prediction,
					feature_contributions: None,
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, features, feature_contributions) in zip!(
					outputs.iter_mut(),
					features.axis_iter(Axis(0)),
					feature_contributions,
				) {
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().cloned(),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					output.feature_contributions = Some(FeatureContributions {
						baseline_value,
						output_value,
						entries: feature_contributions,
					});
				}
			}
			outputs
		}
		RegressionModel::Tree(inner_model) => {
			let features = modelfox_features::compute_features_array_value(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut predictions = Array::zeros(n_rows);
			inner_model.predict(features.view(), predictions.view_mut());
			let mut outputs: Vec<RegressionPredictOutput> = predictions
				.iter()
				.map(|prediction| RegressionPredictOutput {
					value: *prediction,
					feature_contributions: None,
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, features, feature_contributions) in zip!(
					outputs.iter_mut(),
					features.axis_iter(Axis(0)),
					feature_contributions,
				) {
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().map(|v| match v {
							modelfox_table::TableValue::Number(value) => *value,
							modelfox_table::TableValue::Enum(value) => {
								value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
							}
							_ => unreachable!(),
						}),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					output.feature_contributions = Some(FeatureContributions {
						baseline_value,
						output_value,
						entries: feature_contributions,
					});
				}
			}
			outputs
		}
	}
}

fn predict_binary_classifier(
	model: &BinaryClassifier,
	table: Table,
	options: &PredictOptions,
) -> Vec<BinaryClassificationPredictOutput> {
	let n_rows = table.nrows();
	match &model.model {
		BinaryClassificationModel::Linear(inner_model) => {
			let mut probabilities = Array::zeros(n_rows);
			let features = modelfox_features::compute_features_array_f32(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), probabilities.view_mut());
			let mut outputs: Vec<BinaryClassificationPredictOutput> = probabilities
				.iter()
				.map(|probability| {
					let (probability, class_name) = if *probability >= options.threshold {
						(*probability, model.positive_class.clone())
					} else {
						(1.0 - probability, model.negative_class.clone())
					};
					BinaryClassificationPredictOutput {
						class_name,
						probability,
						feature_contributions: None,
					}
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, feature_contributions) in
					zip!(outputs.iter_mut(), feature_contributions)
				{
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().cloned(),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					output.feature_contributions = Some(FeatureContributions {
						baseline_value,
						output_value,
						entries: feature_contributions,
					});
				}
			}
			outputs
		}
		BinaryClassificationModel::Tree(inner_model) => {
			let features = modelfox_features::compute_features_array_value(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut probabilities = Array::zeros(n_rows);
			inner_model.predict(features.view(), probabilities.view_mut());
			let mut outputs: Vec<BinaryClassificationPredictOutput> = probabilities
				.iter()
				.map(|probability| {
					let (probability, class_name) = if *probability >= options.threshold {
						(*probability, model.positive_class.clone())
					} else {
						(1.0 - probability, model.negative_class.clone())
					};
					BinaryClassificationPredictOutput {
						class_name,
						probability,
						feature_contributions: None,
					}
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, feature_contributions) in
					zip!(outputs.iter_mut(), feature_contributions)
				{
					let baseline_value = feature_contributions.baseline_value;
					let output_value = feature_contributions.output_value;
					let feature_contributions = compute_feature_contributions(
						model.feature_groups.iter(),
						features.iter().map(|v| match v {
							modelfox_table::TableValue::Number(value) => *value,
							modelfox_table::TableValue::Enum(value) => {
								value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
							}
							_ => unreachable!(),
						}),
						feature_contributions
							.feature_contribution_values
							.into_iter(),
					);
					output.feature_contributions = Some(FeatureContributions {
						baseline_value,
						output_value,
						entries: feature_contributions,
					});
				}
			}
			outputs
		}
	}
}

fn predict_multiclass_classifier(
	model: &MulticlassClassifier,
	table: Table,
	options: &PredictOptions,
) -> Vec<MulticlassClassificationPredictOutput> {
	let n_rows = table.nrows();
	let n_classes = model.classes.len();
	match &model.model {
		MulticlassClassificationModel::Linear(inner_model) => {
			let mut probabilities = Array::zeros((n_rows, n_classes));
			let features = modelfox_features::compute_features_array_f32(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			inner_model.predict(features.view(), probabilities.view_mut());
			let mut outputs: Vec<MulticlassClassificationPredictOutput> = probabilities
				.axis_iter(Axis(0))
				.map(|probabilities| {
					let (probability, class_name) =
						zip!(probabilities.iter(), model.classes.iter())
							.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
							.unwrap();
					let probabilities = zip!(probabilities, model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect();
					MulticlassClassificationPredictOutput {
						class_name: class_name.clone(),
						probability: *probability,
						probabilities,
						feature_contributions: None,
					}
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, feature_contributions) in
					zip!(outputs.iter_mut(), feature_contributions)
				{
					let feature_contributions = zip!(model.classes.iter(), feature_contributions)
						.map(|(class, feature_contributions)| {
							let baseline_value = feature_contributions.baseline_value;
							let output_value = feature_contributions.output_value;
							let feature_contributions = compute_feature_contributions(
								model.feature_groups.iter(),
								features.iter().cloned(),
								feature_contributions
									.feature_contribution_values
									.into_iter(),
							);
							let feature_contributions = FeatureContributions {
								baseline_value,
								output_value,
								entries: feature_contributions,
							};
							(class.clone(), feature_contributions)
						})
						.collect();
					output.feature_contributions = Some(feature_contributions)
				}
			}
			outputs
		}
		MulticlassClassificationModel::Tree(inner_model) => {
			let features = modelfox_features::compute_features_array_value(
				&table.view(),
				&model.feature_groups,
				&|| {},
			);
			let mut probabilities = Array::zeros((n_rows, n_classes));
			inner_model.predict(features.view(), probabilities.view_mut());
			let mut outputs: Vec<MulticlassClassificationPredictOutput> = probabilities
				.axis_iter(Axis(0))
				.map(|probabilities| {
					let (probability, class_name) =
						zip!(probabilities.iter(), model.classes.iter())
							.max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
							.unwrap();
					let probabilities = zip!(probabilities, model.classes.iter())
						.map(|(p, c)| (c.clone(), *p))
						.collect();
					MulticlassClassificationPredictOutput {
						class_name: class_name.clone(),
						probability: *probability,
						probabilities,
						feature_contributions: None,
					}
				})
				.collect();
			if options.compute_feature_contributions {
				let feature_contributions =
					inner_model.compute_feature_contributions(features.view());
				for (mut output, feature_contributions) in
					zip!(outputs.iter_mut(), feature_contributions)
				{
					let feature_contributions = zip!(model.classes.iter(), feature_contributions)
						.map(|(class, feature_contributions)| {
							let baseline_value = feature_contributions.baseline_value;
							let output_value = feature_contributions.output_value;
							let feature_contributions = compute_feature_contributions(
								model.feature_groups.iter(),
								features.iter().map(|v| match v {
									modelfox_table::TableValue::Number(value) => *value,
									modelfox_table::TableValue::Enum(value) => {
										value.map(|v| v.get()).unwrap_or(0).to_f32().unwrap()
									}
									_ => unreachable!(),
								}),
								feature_contributions
									.feature_contribution_values
									.into_iter(),
							);
							let feature_contributions = FeatureContributions {
								baseline_value,
								output_value,
								entries: feature_contributions,
							};
							(class.clone(), feature_contributions)
						})
						.collect();
					output.feature_contributions = Some(feature_contributions)
				}
			}
			outputs
		}
	}
}

fn compute_feature_contributions<'a>(
	feature_groups: impl Iterator<Item = &'a modelfox_features::FeatureGroup>,
	mut features: impl Iterator<Item = f32>,
	mut feature_contribution_values: impl Iterator<Item = f32>,
) -> Vec<FeatureContributionEntry> {
	let mut entries = Vec::new();
	for feature_group in feature_groups {
		match feature_group {
			modelfox_features::FeatureGroup::Identity(feature_group) => {
				let feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				entries.push(FeatureContributionEntry::Identity(
					IdentityFeatureContribution {
						column_name: feature_group.source_column_name.clone(),
						feature_contribution_value,
						feature_value,
					},
				));
			}
			modelfox_features::FeatureGroup::Normalized(feature_group) => {
				let feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				entries.push(FeatureContributionEntry::Normalized(
					NormalizedFeatureContribution {
						column_name: feature_group.source_column_name.clone(),
						feature_value,
						feature_contribution_value,
					},
				));
			}
			modelfox_features::FeatureGroup::OneHotEncoded(feature_group) => {
				let feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				entries.push(FeatureContributionEntry::OneHotEncoded(
					OneHotEncodedFeatureContribution {
						column_name: feature_group.source_column_name.clone(),
						variant: None,
						feature_value: feature_value > 0.0,
						feature_contribution_value,
					},
				));
				for variant in feature_group.variants.iter() {
					let feature_value = features.next().unwrap();
					let feature_contribution_value = feature_contribution_values.next().unwrap();
					entries.push(FeatureContributionEntry::OneHotEncoded(
						OneHotEncodedFeatureContribution {
							column_name: feature_group.source_column_name.clone(),
							variant: Some(variant.clone()),
							feature_value: feature_value > 0.0,
							feature_contribution_value,
						},
					));
				}
			}
			modelfox_features::FeatureGroup::BagOfWords(feature_group) => {
				for ngram in feature_group.ngrams.keys() {
					let feature_value = features.next().unwrap();
					let feature_contribution_value = feature_contribution_values.next().unwrap();
					entries.push(FeatureContributionEntry::BagOfWords(
						BagOfWordsFeatureContribution {
							column_name: feature_group.source_column_name.clone(),
							ngram: ngram.clone().into(),
							feature_value,
							feature_contribution_value,
						},
					));
				}
			}
			modelfox_features::FeatureGroup::BagOfWordsCosineSimilarity(feature_group) => {
				let feature_value = features.next().unwrap();
				let feature_contribution_value = feature_contribution_values.next().unwrap();
				entries.push(FeatureContributionEntry::BagOfWordsCosineSimilarity(
					BagOfWordsCosineSimilarityFeatureContribution {
						column_name_a: feature_group.source_column_name_a.clone(),
						column_name_b: feature_group.source_column_name_b.clone(),
						feature_value,
						feature_contribution_value,
					},
				));
			}
			modelfox_features::FeatureGroup::WordEmbedding(feature_group) => {
				for value_index in 0..feature_group.model.size {
					let feature_contribution_value = feature_contribution_values.next().unwrap();
					entries.push(FeatureContributionEntry::WordEmbedding(
						WordEmbeddingFeatureContribution {
							column_name: feature_group.source_column_name.clone(),
							value_index,
							feature_contribution_value,
						},
					));
				}
			}
		}
	}
	entries
}
