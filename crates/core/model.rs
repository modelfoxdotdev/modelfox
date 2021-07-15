use crate::{
	stats::{
		ColumnStatsOutput, EnumColumnStatsOutput, NumberColumnStatsOutput, StatsSettings,
		TextColumnStatsOutput, TextColumnStatsOutputTopNGramsEntry, UnknownColumnStatsOutput,
	},
	train::{TrainGridItemOutput, TrainModelOutput},
};
use anyhow::Result;
use num::ToPrimitive;
use std::path::Path;
use tangram_id::Id;
use tangram_zip::zip;

pub struct Model {
	pub id: Id,
	pub version: String,
	pub date: String,
	pub inner: ModelInner,
}

pub enum ModelInner {
	Regressor(Regressor),
	BinaryClassifier(BinaryClassifier),
	MulticlassClassifier(MulticlassClassifier),
}

pub struct Regressor {
	pub target_column_name: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
	pub overall_row_count: usize,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStatsOutput>,
	pub overall_target_column_stats: ColumnStatsOutput,
	pub train_column_stats: Vec<ColumnStatsOutput>,
	pub train_target_column_stats: ColumnStatsOutput,
	pub test_column_stats: Vec<ColumnStatsOutput>,
	pub test_target_column_stats: ColumnStatsOutput,
	pub baseline_metrics: tangram_metrics::RegressionMetricsOutput,
	pub comparison_metric: RegressionComparisonMetric,
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	pub best_grid_item_index: usize,
	pub model: RegressionModel,
	pub test_metrics: tangram_metrics::RegressionMetricsOutput,
}

pub struct BinaryClassifier {
	pub target_column_name: String,
	pub negative_class: String,
	pub positive_class: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
	pub overall_row_count: usize,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStatsOutput>,
	pub overall_target_column_stats: ColumnStatsOutput,
	pub train_column_stats: Vec<ColumnStatsOutput>,
	pub train_target_column_stats: ColumnStatsOutput,
	pub test_column_stats: Vec<ColumnStatsOutput>,
	pub test_target_column_stats: ColumnStatsOutput,
	pub baseline_metrics: tangram_metrics::BinaryClassificationMetricsOutput,
	pub comparison_metric: BinaryClassificationComparisonMetric,
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	pub best_grid_item_index: usize,
	pub model: BinaryClassificationModel,
	pub test_metrics: tangram_metrics::BinaryClassificationMetricsOutput,
}

pub struct MulticlassClassifier {
	pub target_column_name: String,
	pub classes: Vec<String>,
	pub train_row_count: usize,
	pub test_row_count: usize,
	pub overall_row_count: usize,
	pub stats_settings: StatsSettings,
	pub overall_column_stats: Vec<ColumnStatsOutput>,
	pub overall_target_column_stats: ColumnStatsOutput,
	pub train_column_stats: Vec<ColumnStatsOutput>,
	pub train_target_column_stats: ColumnStatsOutput,
	pub test_column_stats: Vec<ColumnStatsOutput>,
	pub test_target_column_stats: ColumnStatsOutput,
	pub baseline_metrics: tangram_metrics::MulticlassClassificationMetricsOutput,
	pub comparison_metric: MulticlassClassificationComparisonMetric,
	pub train_grid_item_outputs: Vec<TrainGridItemOutput>,
	pub best_grid_item_index: usize,
	pub model: MulticlassClassificationModel,
	pub test_metrics: tangram_metrics::MulticlassClassificationMetricsOutput,
}

#[derive(Clone, Copy)]
pub enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification,
}

#[derive(Clone, Copy)]
pub enum BinaryClassificationComparisonMetric {
	AucRoc,
}

#[derive(Clone, Copy)]
pub enum MulticlassClassificationComparisonMetric {
	Accuracy,
}

pub enum RegressionModel {
	Linear(LinearRegressionModel),
	Tree(TreeRegressionModel),
}

pub struct LinearRegressionModel {
	pub model: tangram_linear::Regressor,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

pub struct TreeRegressionModel {
	pub model: tangram_tree::Regressor,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Copy)]
pub enum RegressionComparisonMetric {
	MeanAbsoluteError,
	MeanSquaredError,
	RootMeanSquaredError,
	R2,
}

pub enum BinaryClassificationModel {
	Linear(LinearBinaryClassificationModel),
	Tree(TreeBinaryClassificationModel),
}

pub struct LinearBinaryClassificationModel {
	pub model: tangram_linear::BinaryClassifier,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

pub struct TreeBinaryClassificationModel {
	pub model: tangram_tree::BinaryClassifier,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

pub enum MulticlassClassificationModel {
	Linear(LinearMulticlassClassificationModel),
	Tree(TreeMulticlassClassificationModel),
}

pub struct LinearMulticlassClassificationModel {
	pub model: tangram_linear::MulticlassClassifier,
	pub train_options: tangram_linear::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

pub struct TreeMulticlassClassificationModel {
	pub model: tangram_tree::MulticlassClassifier,
	pub train_options: tangram_tree::TrainOptions,
	pub feature_groups: Vec<tangram_features::FeatureGroup>,
	pub losses: Option<Vec<f32>>,
	pub feature_importances: Vec<f32>,
}

#[derive(Clone, Copy)]
pub enum ComparisonMetric {
	Regression(RegressionComparisonMetric),
	BinaryClassification(BinaryClassificationComparisonMetric),
	MulticlassClassification(MulticlassClassificationComparisonMetric),
}

pub enum Metrics {
	Regression(tangram_metrics::RegressionMetricsOutput),
	BinaryClassification(tangram_metrics::BinaryClassificationMetricsOutput),
	MulticlassClassification(tangram_metrics::MulticlassClassificationMetricsOutput),
}

impl Model {
	pub fn to_path(&self, path: &Path) -> Result<()> {
		let mut writer = buffalo::Writer::new();
		let model = serialize_model(self, &mut writer);
		writer.write(&model);
		let bytes = writer.into_bytes();
		tangram_model::to_path(path, &bytes)?;
		Ok(())
	}
}

fn serialize_model(
	model: &Model,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::ModelWriter> {
	let id = writer.write(model.id.to_string().as_str());
	let version = writer.write(model.version.as_str());
	let date = writer.write(model.date.to_string().as_str());
	let inner = serialize_model_inner(&model.inner, writer);
	writer.write(&tangram_model::ModelWriter {
		id,
		version,
		date,
		inner,
	})
}

fn serialize_model_inner(
	model_inner: &ModelInner,
	writer: &mut buffalo::Writer,
) -> tangram_model::ModelInnerWriter {
	match model_inner {
		ModelInner::Regressor(regressor) => {
			let regressor = serialize_regressor(regressor, writer);
			tangram_model::ModelInnerWriter::Regressor(regressor)
		}
		ModelInner::BinaryClassifier(binary_classifier) => {
			let binary_classifier = serialize_binary_classifier(binary_classifier, writer);
			tangram_model::ModelInnerWriter::BinaryClassifier(binary_classifier)
		}
		ModelInner::MulticlassClassifier(multiclass_classifier) => {
			let multiclass_classifier =
				serialize_multiclass_classifier(multiclass_classifier, writer);
			tangram_model::ModelInnerWriter::MulticlassClassifier(multiclass_classifier)
		}
	}
}

fn serialize_regressor(
	regressor: &Regressor,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::RegressorWriter> {
	let target_column_name = writer.write(regressor.target_column_name.as_str());
	let stats_settings = serialize_stats_settings(&regressor.stats_settings, writer);
	let overall_column_stats = regressor
		.overall_column_stats
		.iter()
		.map(|overall_column_stats| serialize_column_stats_output(overall_column_stats, writer))
		.collect::<Vec<_>>();
	let overall_column_stats = writer.write(&overall_column_stats);
	let overall_target_column_stats =
		serialize_column_stats_output(&regressor.overall_target_column_stats, writer);
	let train_column_stats = regressor
		.train_column_stats
		.iter()
		.map(|train_column_stats| serialize_column_stats_output(&train_column_stats, writer))
		.collect::<Vec<_>>();
	let train_column_stats = writer.write(&train_column_stats);
	let train_target_column_stats =
		serialize_column_stats_output(&regressor.train_target_column_stats, writer);
	let test_column_stats = regressor
		.test_column_stats
		.iter()
		.map(|test_column_stats| serialize_column_stats_output(test_column_stats, writer))
		.collect::<Vec<_>>();
	let test_column_stats = writer.write(&test_column_stats);
	let test_target_column_stats =
		serialize_column_stats_output(&regressor.test_target_column_stats, writer);
	let baseline_metrics = serialize_regression_metrics_output(&regressor.baseline_metrics, writer);
	let comparison_metric =
		serialize_regression_comparison_metric(&regressor.comparison_metric, writer);
	let train_grid_item_outputs = regressor
		.train_grid_item_outputs
		.iter()
		.map(|train_grid_item_output| {
			serialize_train_grid_item_output(train_grid_item_output, writer)
		})
		.collect::<Vec<_>>();
	let train_grid_item_outputs = writer.write(&train_grid_item_outputs);
	let model = serialize_regression_model(&regressor.model, writer);
	let test_metrics = serialize_regression_metrics_output(&regressor.test_metrics, writer);
	let regressor_writer = tangram_model::RegressorWriter {
		target_column_name,
		train_row_count: regressor.train_row_count.to_u64().unwrap(),
		test_row_count: regressor.test_row_count.to_u64().unwrap(),
		overall_row_count: regressor.overall_row_count.to_u64().unwrap(),
		stats_settings,
		overall_column_stats,
		overall_target_column_stats,
		train_column_stats,
		train_target_column_stats,
		test_column_stats,
		test_target_column_stats,
		baseline_metrics,
		comparison_metric,
		train_grid_item_outputs,
		best_grid_item_index: regressor.best_grid_item_index.to_u64().unwrap(),
		model,
		test_metrics,
	};
	writer.write(&regressor_writer)
}

fn serialize_binary_classifier(
	binary_classifier: &BinaryClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BinaryClassifierWriter> {
	let negative_class = writer.write(binary_classifier.negative_class.as_str());
	let positive_class = writer.write(binary_classifier.positive_class.as_str());
	let target_column_name = writer.write(binary_classifier.target_column_name.as_str());
	let stats_settings = serialize_stats_settings(&binary_classifier.stats_settings, writer);
	let overall_column_stats = binary_classifier
		.overall_column_stats
		.iter()
		.map(|overall_column_stats| serialize_column_stats_output(overall_column_stats, writer))
		.collect::<Vec<_>>();
	let overall_column_stats = writer.write(&overall_column_stats);
	let overall_target_column_stats =
		serialize_column_stats_output(&binary_classifier.overall_target_column_stats, writer);
	let train_column_stats = binary_classifier
		.train_column_stats
		.iter()
		.map(|train_column_stats| serialize_column_stats_output(&train_column_stats, writer))
		.collect::<Vec<_>>();
	let train_column_stats = writer.write(&train_column_stats);
	let train_target_column_stats =
		serialize_column_stats_output(&binary_classifier.train_target_column_stats, writer);
	let test_column_stats = binary_classifier
		.test_column_stats
		.iter()
		.map(|test_column_stats| serialize_column_stats_output(&test_column_stats, writer))
		.collect::<Vec<_>>();
	let test_column_stats = writer.write(&test_column_stats);
	let test_target_column_stats =
		serialize_column_stats_output(&binary_classifier.test_target_column_stats, writer);
	let baseline_metrics =
		serialize_binary_classification_metrics_output(&binary_classifier.baseline_metrics, writer);
	let comparison_metric = serialize_binary_classification_comparison_metric(
		&binary_classifier.comparison_metric,
		writer,
	);
	let train_grid_item_outputs = binary_classifier
		.train_grid_item_outputs
		.iter()
		.map(|train_grid_item_output| {
			serialize_train_grid_item_output(train_grid_item_output, writer)
		})
		.collect::<Vec<_>>();
	let train_grid_item_outputs = writer.write(&train_grid_item_outputs);
	let model = serialize_binary_classification_model(&binary_classifier.model, writer);
	let test_metrics =
		serialize_binary_classification_metrics_output(&binary_classifier.test_metrics, writer);
	let binary_classifier_writer = tangram_model::BinaryClassifierWriter {
		target_column_name,
		train_row_count: binary_classifier.train_row_count.to_u64().unwrap(),
		test_row_count: binary_classifier.test_row_count.to_u64().unwrap(),
		overall_row_count: binary_classifier.overall_row_count.to_u64().unwrap(),
		stats_settings,
		overall_column_stats,
		overall_target_column_stats,
		train_column_stats,
		train_target_column_stats,
		test_column_stats,
		test_target_column_stats,
		baseline_metrics,
		comparison_metric,
		train_grid_item_outputs,
		best_grid_item_index: binary_classifier.best_grid_item_index.to_u64().unwrap(),
		model,
		test_metrics,
		negative_class,
		positive_class,
	};
	writer.write(&binary_classifier_writer)
}

fn serialize_multiclass_classifier(
	multiclass_classifier: &MulticlassClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::MulticlassClassifierWriter> {
	let target_column_name = writer.write(multiclass_classifier.target_column_name.as_str());
	let stats_settings = serialize_stats_settings(&multiclass_classifier.stats_settings, writer);
	let overall_column_stats = multiclass_classifier
		.overall_column_stats
		.iter()
		.map(|overall_column_stats| serialize_column_stats_output(overall_column_stats, writer))
		.collect::<Vec<_>>();
	let overall_column_stats = writer.write(&overall_column_stats);
	let overall_target_column_stats =
		serialize_column_stats_output(&multiclass_classifier.overall_target_column_stats, writer);
	let train_column_stats = multiclass_classifier
		.train_column_stats
		.iter()
		.map(|train_column_stats| serialize_column_stats_output(train_column_stats, writer))
		.collect::<Vec<_>>();
	let train_column_stats = writer.write(&train_column_stats);
	let train_target_column_stats =
		serialize_column_stats_output(&multiclass_classifier.train_target_column_stats, writer);
	let test_column_stats = multiclass_classifier
		.test_column_stats
		.iter()
		.map(|test_column_stats| serialize_column_stats_output(test_column_stats, writer))
		.collect::<Vec<_>>();
	let test_column_stats = writer.write(&test_column_stats);
	let test_target_column_stats =
		serialize_column_stats_output(&multiclass_classifier.test_target_column_stats, writer);
	let baseline_metrics = serialize_multiclass_classification_metrics_output(
		&multiclass_classifier.baseline_metrics,
		writer,
	);
	let comparison_metric = serialize_multiclass_classification_comparison_metric(
		&multiclass_classifier.comparison_metric,
		writer,
	);
	let train_grid_item_outputs = multiclass_classifier
		.train_grid_item_outputs
		.iter()
		.map(|train_grid_item_output| {
			serialize_train_grid_item_output(train_grid_item_output, writer)
		})
		.collect::<Vec<_>>();
	let train_grid_item_outputs = writer.write(&train_grid_item_outputs);
	let model = serialize_multiclass_classification_model(&multiclass_classifier.model, writer);
	let test_metrics = serialize_multiclass_classification_metrics_output(
		&multiclass_classifier.test_metrics,
		writer,
	);
	let classes = multiclass_classifier
		.classes
		.iter()
		.map(|class| writer.write(class))
		.collect::<Vec<_>>();
	let classes = writer.write(&classes);
	let multiclass_classifier_writer = tangram_model::MulticlassClassifierWriter {
		target_column_name,
		train_row_count: multiclass_classifier.train_row_count.to_u64().unwrap(),
		test_row_count: multiclass_classifier.test_row_count.to_u64().unwrap(),
		overall_row_count: multiclass_classifier.overall_row_count.to_u64().unwrap(),
		stats_settings,
		overall_column_stats,
		overall_target_column_stats,
		train_column_stats,
		train_target_column_stats,
		test_column_stats,
		test_target_column_stats,
		baseline_metrics,
		comparison_metric,
		train_grid_item_outputs,
		best_grid_item_index: multiclass_classifier.best_grid_item_index.to_u64().unwrap(),
		model,
		test_metrics,
		classes,
	};
	writer.write(&multiclass_classifier_writer)
}

fn serialize_stats_settings(
	stats_settings: &StatsSettings,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::StatsSettingsWriter> {
	let stats_settings_writer = tangram_model::StatsSettingsWriter {
		number_histogram_max_size: stats_settings.number_histogram_max_size.to_u64().unwrap(),
	};
	writer.write(&stats_settings_writer)
}

fn serialize_column_stats_output(
	column_stats_output: &ColumnStatsOutput,
	writer: &mut buffalo::Writer,
) -> tangram_model::ColumnStatsWriter {
	match column_stats_output {
		ColumnStatsOutput::Unknown(unknown_column_stats) => {
			let unknown_column_stats =
				serialize_unknown_column_stats_output(&unknown_column_stats, writer);
			tangram_model::ColumnStatsWriter::UnknownColumn(unknown_column_stats)
		}
		ColumnStatsOutput::Number(number_column_stats) => {
			let number_column_stats =
				serialize_number_column_stats_output(&number_column_stats, writer);
			tangram_model::ColumnStatsWriter::NumberColumn(number_column_stats)
		}
		ColumnStatsOutput::Enum(enum_column_stats) => {
			let enum_column_stats = serialize_enum_column_stats_output(&enum_column_stats, writer);
			tangram_model::ColumnStatsWriter::EnumColumn(enum_column_stats)
		}
		ColumnStatsOutput::Text(text_column_stats) => {
			let text_column_stats = serialize_text_column_stats_output(&text_column_stats, writer);
			tangram_model::ColumnStatsWriter::TextColumn(text_column_stats)
		}
	}
}

fn serialize_unknown_column_stats_output(
	uknown_column_stats_output: &UnknownColumnStatsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::UnknownColumnStatsWriter> {
	let column_name = writer.write(uknown_column_stats_output.column_name.as_str());
	let unknown_column_stats = tangram_model::UnknownColumnStatsWriter { column_name };
	writer.write(&unknown_column_stats)
}

fn serialize_number_column_stats_output(
	number_column_stats_output: &NumberColumnStatsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::NumberColumnStatsWriter> {
	let column_name = writer.write(number_column_stats_output.column_name.as_str());
	let histogram = number_column_stats_output
		.histogram
		.as_ref()
		.map(|histogram| {
			histogram
				.iter()
				.map(|(key, value)| (key.get(), value.to_u64().unwrap()))
				.collect::<Vec<_>>()
		});
	let histogram = histogram.map(|histogram| writer.write(histogram.as_slice()));
	let number_column_stats = tangram_model::NumberColumnStatsWriter {
		column_name,
		invalid_count: number_column_stats_output.invalid_count.to_u64().unwrap(),
		unique_count: number_column_stats_output.unique_count.to_u64().unwrap(),
		histogram,
		min: number_column_stats_output.min,
		max: number_column_stats_output.max,
		mean: number_column_stats_output.mean,
		variance: number_column_stats_output.variance,
		std: number_column_stats_output.std,
		p25: number_column_stats_output.p25,
		p50: number_column_stats_output.p50,
		p75: number_column_stats_output.p75,
	};
	writer.write(&number_column_stats)
}

fn serialize_enum_column_stats_output(
	enum_column_stats_output: &EnumColumnStatsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::EnumColumnStatsWriter> {
	let column_name = writer.write(enum_column_stats_output.column_name.as_str());
	let strings = enum_column_stats_output
		.histogram
		.iter()
		.map(|(key, _)| writer.write(key))
		.collect::<Vec<_>>();
	let histogram = zip!(strings, enum_column_stats_output.histogram.iter())
		.map(|(key, (_, value))| (key, value.to_u64().unwrap()))
		.collect::<Vec<_>>();
	let histogram = writer.write(&histogram);
	let enum_column_stats = tangram_model::EnumColumnStatsWriter {
		column_name,
		invalid_count: enum_column_stats_output.invalid_count.to_u64().unwrap(),
		histogram,
		unique_count: enum_column_stats_output.unique_count.to_u64().unwrap(),
	};
	writer.write(&enum_column_stats)
}

fn serialize_text_column_stats_output(
	text_column_stats_output: &TextColumnStatsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TextColumnStatsWriter> {
	let column_name = writer.write(text_column_stats_output.column_name.as_str());
	let tokenizer = serialize_tokenizer(&text_column_stats_output.tokenizer, writer);
	let ngram_types = text_column_stats_output
		.ngram_types
		.iter()
		.map(|ngram_type| serialize_ngram_type(ngram_type, writer))
		.collect::<Vec<_>>();
	let ngram_types = writer.write(&ngram_types);
	let ngrams_count = text_column_stats_output.ngrams_count.to_u64().unwrap();
	let top_ngrams = text_column_stats_output
		.top_ngrams
		.iter()
		.map(|(ngram, entry)| {
			(
				serialize_ngram(&ngram, writer),
				serialize_text_column_stats_output_top_n_grams_entry(entry, writer),
			)
		})
		.collect::<Vec<_>>();
	let top_ngrams = writer.write(&top_ngrams);
	let text_column_stats = tangram_model::TextColumnStatsWriter {
		column_name,
		tokenizer,
		ngram_types,
		ngrams_count,
		top_ngrams,
	};
	writer.write(&text_column_stats)
}

fn serialize_tokenizer(
	tokenizer: &tangram_text::Tokenizer,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TokenizerWriter> {
	writer.write(&tangram_model::TokenizerWriter {
		lowercase: tokenizer.lowercase,
		alphanumeric: tokenizer.alphanumeric,
	})
}

fn serialize_ngram(
	ngram: &tangram_text::NGram,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::NGramWriter> {
	match ngram {
		tangram_text::NGram::Unigram(token) => {
			let token = writer.write(token);
			writer.write(&tangram_model::NGramWriter::Unigram(token))
		}
		tangram_text::NGram::Bigram(token_a, token_b) => {
			let token_a = writer.write(token_a);
			let token_b = writer.write(token_b);
			writer.write(&tangram_model::NGramWriter::Bigram((token_a, token_b)))
		}
	}
}

fn serialize_text_column_stats_output_top_n_grams_entry(
	text_column_stats_output_top_n_grams_entry: &TextColumnStatsOutputTopNGramsEntry,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TextColumnStatsTopNGramsEntryWriter> {
	let token_stats = tangram_model::TextColumnStatsTopNGramsEntryWriter {
		occurrence_count: text_column_stats_output_top_n_grams_entry
			.occurrence_count
			.to_u64()
			.unwrap(),
		row_count: text_column_stats_output_top_n_grams_entry
			.row_count
			.to_u64()
			.unwrap(),
	};
	writer.write(&token_stats)
}

fn serialize_ngram_type(
	ngram_type: &tangram_text::NGramType,
	_writer: &mut buffalo::Writer,
) -> tangram_model::NGramTypeWriter {
	match ngram_type {
		tangram_text::NGramType::Unigram => tangram_model::NGramTypeWriter::Unigram,
		tangram_text::NGramType::Bigram => tangram_model::NGramTypeWriter::Bigram,
	}
}

fn serialize_regression_metrics_output(
	regression_metrics_output: &tangram_metrics::RegressionMetricsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::RegressionMetricsWriter> {
	let regression_metrics_writer = tangram_model::RegressionMetricsWriter {
		mse: regression_metrics_output.mse,
		rmse: regression_metrics_output.rmse,
		mae: regression_metrics_output.mae,
		r2: regression_metrics_output.r2,
	};
	writer.write(&regression_metrics_writer)
}

fn serialize_regression_comparison_metric(
	regression_comparison_metric_writer: &RegressionComparisonMetric,
	_writer: &mut buffalo::Writer,
) -> tangram_model::RegressionComparisonMetricWriter {
	match regression_comparison_metric_writer {
		RegressionComparisonMetric::MeanAbsoluteError => {
			tangram_model::RegressionComparisonMetricWriter::MeanAbsoluteError
		}
		RegressionComparisonMetric::MeanSquaredError => {
			tangram_model::RegressionComparisonMetricWriter::MeanSquaredError
		}
		RegressionComparisonMetric::RootMeanSquaredError => {
			tangram_model::RegressionComparisonMetricWriter::RootMeanSquaredError
		}
		RegressionComparisonMetric::R2 => tangram_model::RegressionComparisonMetricWriter::R2,
	}
}

fn serialize_regression_model(
	regression_model: &RegressionModel,
	writer: &mut buffalo::Writer,
) -> tangram_model::RegressionModelWriter {
	match regression_model {
		RegressionModel::Linear(linear_model) => {
			let linear_regressor = serialize_linear_regression_model(linear_model, writer);
			tangram_model::RegressionModelWriter::Linear(linear_regressor)
		}
		RegressionModel::Tree(tree_model) => {
			let tree_regressor = serialize_tree_regression_model(tree_model, writer);
			tangram_model::RegressionModelWriter::Tree(tree_regressor)
		}
	}
}

fn serialize_early_stopping_options(
	early_stopping_options: &tangram_linear::EarlyStoppingOptions,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::LinearEarlyStoppingOptionsWriter> {
	let early_stopping_options = tangram_model::LinearEarlyStoppingOptionsWriter {
		early_stopping_fraction: early_stopping_options.early_stopping_fraction,
		n_rounds_without_improvement_to_stop: early_stopping_options
			.n_rounds_without_improvement_to_stop
			.to_u64()
			.unwrap(),
		min_decrease_in_loss_for_significant_change: early_stopping_options
			.min_decrease_in_loss_for_significant_change,
	};
	writer.write(&early_stopping_options)
}

fn serialize_linear_regression_model(
	linear_regression_model: &LinearRegressionModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::LinearRegressorWriter> {
	let feature_importances = writer.write(linear_regression_model.feature_importances.as_slice());
	let train_options =
		serialize_linear_train_options(&linear_regression_model.train_options, writer);
	let feature_groups = linear_regression_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = linear_regression_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let model = linear_regression_model.model.to_writer(writer);
	let linear_regressor_writer = tangram_model::LinearRegressorWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&linear_regressor_writer)
}

fn serialize_linear_train_options(
	train_options: &tangram_linear::TrainOptions,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::LinearModelTrainOptionsWriter> {
	let early_stopping_options =
		train_options
			.early_stopping_options
			.as_ref()
			.map(|early_stopping_options| {
				serialize_early_stopping_options(early_stopping_options, writer)
			});
	let train_options = tangram_model::LinearModelTrainOptionsWriter {
		compute_loss: train_options.compute_losses,
		l2_regularization: train_options.l2_regularization,
		learning_rate: train_options.learning_rate,
		max_epochs: train_options.max_epochs.to_u64().unwrap(),
		n_examples_per_batch: train_options.n_examples_per_batch.to_u64().unwrap(),
		early_stopping_options,
	};
	writer.write(&train_options)
}

fn serialize_tree_train_options(
	train_options: &tangram_tree::TrainOptions,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TreeModelTrainOptionsWriter> {
	let early_stopping_options =
		train_options
			.early_stopping_options
			.as_ref()
			.map(|early_stopping_options| {
				serialize_tree_early_stopping_options(early_stopping_options, writer)
			});
	let max_depth = train_options
		.max_depth
		.map(|max_depth| max_depth.to_u64().unwrap());
	let binned_features_layout =
		serialize_binned_features_layout(&train_options.binned_features_layout, writer);
	let train_options = tangram_model::TreeModelTrainOptionsWriter {
		compute_loss: train_options.compute_losses,
		l2_regularization_for_continuous_splits: train_options
			.l2_regularization_for_continuous_splits,
		l2_regularization_for_discrete_splits: train_options.l2_regularization_for_discrete_splits,
		learning_rate: train_options.learning_rate,
		early_stopping_options,
		binned_features_layout,
		max_depth,
		max_examples_for_computing_bin_thresholds: train_options
			.max_examples_for_computing_bin_thresholds
			.to_u64()
			.unwrap(),
		max_leaf_nodes: train_options.max_leaf_nodes.to_u64().unwrap(),
		max_rounds: train_options.max_rounds.to_u64().unwrap(),
		max_valid_bins_for_number_features: train_options
			.max_valid_bins_for_number_features
			.to_u8()
			.unwrap(),
		min_examples_per_node: train_options.min_examples_per_node.to_u64().unwrap(),
		min_gain_to_split: train_options.min_gain_to_split,
		min_sum_hessians_per_node: train_options.min_sum_hessians_per_node,
		smoothing_factor_for_discrete_bin_sorting: train_options
			.smoothing_factor_for_discrete_bin_sorting,
	};
	writer.write(&train_options)
}

fn serialize_tree_early_stopping_options(
	early_stopping_options: &tangram_tree::EarlyStoppingOptions,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TreeEarlyStoppingOptionsWriter> {
	let early_stopping_options = tangram_model::TreeEarlyStoppingOptionsWriter {
		early_stopping_fraction: early_stopping_options.early_stopping_fraction,
		n_rounds_without_improvement_to_stop: early_stopping_options
			.n_rounds_without_improvement_to_stop
			.to_u64()
			.unwrap(),
		min_decrease_in_loss_for_significant_change: early_stopping_options
			.min_decrease_in_loss_for_significant_change,
	};
	writer.write(&early_stopping_options)
}

fn serialize_tree_regression_model(
	tree_regression_model: &TreeRegressionModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TreeRegressorWriter> {
	let feature_importances = writer.write(tree_regression_model.feature_importances.as_slice());
	let train_options = serialize_tree_train_options(&tree_regression_model.train_options, writer);
	let feature_groups = tree_regression_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = tree_regression_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let model = tree_regression_model.model.to_writer(writer);
	let model = tangram_model::TreeRegressorWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&model)
}

fn serialize_binned_features_layout(
	binned_features_layout: &tangram_tree::BinnedFeaturesLayout,
	_writer: &mut buffalo::Writer,
) -> tangram_model::BinnedFeaturesLayoutWriter {
	match binned_features_layout {
		tangram_tree::BinnedFeaturesLayout::RowMajor => {
			tangram_model::BinnedFeaturesLayoutWriter::RowMajor
		}
		tangram_tree::BinnedFeaturesLayout::ColumnMajor => {
			tangram_model::BinnedFeaturesLayoutWriter::ColumnMajor
		}
	}
}

fn serialize_train_grid_item_output(
	train_grid_item_output: &TrainGridItemOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TrainGridItemOutputWriter> {
	let hyperparameters = match &train_grid_item_output.train_model_output {
		TrainModelOutput::LinearRegressor(model) => {
			let options = serialize_linear_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Linear(options)
		}
		TrainModelOutput::TreeRegressor(model) => {
			let options = serialize_tree_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Tree(options)
		}
		TrainModelOutput::LinearBinaryClassifier(model) => {
			let options = serialize_linear_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Linear(options)
		}
		TrainModelOutput::TreeBinaryClassifier(model) => {
			let options = serialize_tree_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Tree(options)
		}
		TrainModelOutput::LinearMulticlassClassifier(model) => {
			let options = serialize_linear_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Linear(options)
		}
		TrainModelOutput::TreeMulticlassClassifier(model) => {
			let options = serialize_tree_train_options(&model.train_options, writer);
			tangram_model::ModelTrainOptionsWriter::Tree(options)
		}
	};
	let train_grid_item_output_writer = tangram_model::TrainGridItemOutputWriter {
		comparison_metric_value: train_grid_item_output.comparison_metric_value,
		hyperparameters,
		duration: train_grid_item_output.duration.as_secs_f32(),
	};
	writer.write(&train_grid_item_output_writer)
}

fn serialize_feature_group(
	feature_group: &tangram_features::FeatureGroup,
	writer: &mut buffalo::Writer,
) -> tangram_model::FeatureGroupWriter {
	match feature_group {
		tangram_features::FeatureGroup::Identity(feature_group) => {
			let feature_group = serialize_identity_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::Identity(feature_group)
		}
		tangram_features::FeatureGroup::Normalized(feature_group) => {
			let feature_group = serialize_normalized_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::Normalized(feature_group)
		}
		tangram_features::FeatureGroup::OneHotEncoded(feature_group) => {
			let feature_group = serialize_one_hot_encoded_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::OneHotEncoded(feature_group)
		}
		tangram_features::FeatureGroup::BagOfWords(feature_group) => {
			let feature_group = serialize_bag_of_words_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::BagOfWords(feature_group)
		}
		tangram_features::FeatureGroup::BagOfWordsCosineSimilarity(feature_group) => {
			let feature_group =
				serialize_bag_of_words_cosine_similarity_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::BagOfWordsCosineSimilarity(feature_group)
		}
		tangram_features::FeatureGroup::WordEmbedding(feature_group) => {
			let feature_group = serialize_word_embedding_feature_group(feature_group, writer);
			tangram_model::FeatureGroupWriter::WordEmbedding(feature_group)
		}
	}
}

fn serialize_identity_feature_group(
	identity_feature_group: &tangram_features::IdentityFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::IdentityFeatureGroupWriter> {
	let source_column_name = writer.write(identity_feature_group.source_column_name.as_str());
	let feature_group = tangram_model::IdentityFeatureGroupWriter { source_column_name };
	writer.write(&feature_group)
}

fn serialize_normalized_feature_group(
	normalized_feature_group: &tangram_features::NormalizedFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::NormalizedFeatureGroupWriter> {
	let source_column_name = writer.write(normalized_feature_group.source_column_name.as_str());
	let feature_group = tangram_model::NormalizedFeatureGroupWriter {
		source_column_name,
		mean: normalized_feature_group.mean,
		variance: normalized_feature_group.variance,
	};
	writer.write(&feature_group)
}

fn serialize_one_hot_encoded_feature_group(
	one_hot_encoded_feature_group: &tangram_features::OneHotEncodedFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::OneHotEncodedFeatureGroupWriter> {
	let source_column_name =
		writer.write(one_hot_encoded_feature_group.source_column_name.as_str());
	let variants = one_hot_encoded_feature_group
		.variants
		.iter()
		.map(|variant| writer.write(variant))
		.collect::<Vec<_>>();
	let variants = writer.write(&variants);
	let feature_group = tangram_model::OneHotEncodedFeatureGroupWriter {
		source_column_name,
		variants,
	};
	writer.write(&feature_group)
}

fn serialize_bag_of_words_feature_group(
	bag_of_words_feature_group: &tangram_features::BagOfWordsFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BagOfWordsFeatureGroupWriter> {
	let source_column_name = writer.write(bag_of_words_feature_group.source_column_name.as_str());
	let tokenizer = serialize_tokenizer(&bag_of_words_feature_group.tokenizer, writer);
	let ngrams = bag_of_words_feature_group
		.ngrams
		.iter()
		.map(|(ngram, entry)| {
			(
				serialize_ngram(ngram, writer),
				serialize_bag_of_words_feature_group_n_gram_entry(entry, writer),
			)
		})
		.collect::<Vec<_>>();
	let ngrams = writer.write(&ngrams);
	let ngram_types = bag_of_words_feature_group
		.ngram_types
		.iter()
		.map(|ngram_type| serialize_ngram_type(ngram_type, writer))
		.collect::<Vec<_>>();
	let ngram_types = writer.write(&ngram_types);
	let strategy =
		serialize_bag_of_words_feature_group_strategy(&bag_of_words_feature_group.strategy, writer);
	let feature_group = tangram_model::BagOfWordsFeatureGroupWriter {
		source_column_name,
		tokenizer,
		strategy,
		ngram_types,
		ngrams,
	};
	writer.write(&feature_group)
}

fn serialize_bag_of_words_feature_group_strategy(
	bag_of_words_feature_group_strategy: &tangram_features::bag_of_words::BagOfWordsFeatureGroupStrategy,
	_writer: &mut buffalo::Writer,
) -> tangram_model::BagOfWordsFeatureGroupStrategyWriter {
	match bag_of_words_feature_group_strategy {
		tangram_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Present => {
			tangram_model::BagOfWordsFeatureGroupStrategyWriter::Present
		}
		tangram_features::bag_of_words::BagOfWordsFeatureGroupStrategy::Count => {
			tangram_model::BagOfWordsFeatureGroupStrategyWriter::Count
		}
		tangram_features::bag_of_words::BagOfWordsFeatureGroupStrategy::TfIdf => {
			tangram_model::BagOfWordsFeatureGroupStrategyWriter::TfIdf
		}
	}
}

fn serialize_bag_of_words_feature_group_n_gram_entry(
	bag_of_words_feature_group_n_gram_entry: &tangram_features::bag_of_words::BagOfWordsFeatureGroupNGramEntry,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BagOfWordsFeatureGroupNGramEntryWriter> {
	writer.write(&tangram_model::BagOfWordsFeatureGroupNGramEntryWriter {
		idf: bag_of_words_feature_group_n_gram_entry.idf,
	})
}

fn serialize_word_embedding_feature_group(
	word_embedding_feature_group: &tangram_features::WordEmbeddingFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::WordEmbeddingFeatureGroupWriter> {
	let source_column_name = writer.write(word_embedding_feature_group.source_column_name.as_str());
	let tokenizer = serialize_tokenizer(&word_embedding_feature_group.tokenizer, writer);
	let model = serialize_word_embedding_model(&word_embedding_feature_group.model, writer);
	let feature_group = tangram_model::WordEmbeddingFeatureGroupWriter {
		source_column_name,
		tokenizer,
		model,
	};
	writer.write(&feature_group)
}

fn serialize_word_embedding_model(
	word_embedding_model: &tangram_text::WordEmbeddingModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::WordEmbeddingModelWriter> {
	let size = word_embedding_model.size.to_u64().unwrap();
	let values = writer.write(word_embedding_model.values.as_slice());
	let words = word_embedding_model
		.words
		.keys()
		.map(|word| writer.write(word))
		.collect::<Vec<_>>();
	let words = zip!(words, word_embedding_model.words.values())
		.map(|(key, index)| (key, index.to_u64().unwrap()))
		.collect::<Vec<_>>();
	let words = writer.write(&words);
	writer.write(&tangram_model::WordEmbeddingModelWriter {
		size,
		words,
		values,
	})
}

fn serialize_bag_of_words_cosine_similarity_feature_group(
	bag_of_words_cosine_similarity_feature_group: &tangram_features::BagOfWordsCosineSimilarityFeatureGroup,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BagOfWordsCosineSimilarityFeatureGroupWriter> {
	let source_column_name_a = writer.write(
		bag_of_words_cosine_similarity_feature_group
			.source_column_name_a
			.as_str(),
	);
	let source_column_name_b = writer.write(
		bag_of_words_cosine_similarity_feature_group
			.source_column_name_b
			.as_str(),
	);
	let tokenizer = serialize_tokenizer(
		&bag_of_words_cosine_similarity_feature_group.tokenizer,
		writer,
	);
	let ngrams = bag_of_words_cosine_similarity_feature_group
		.ngrams
		.iter()
		.map(|(ngram, entry)| {
			(
				serialize_ngram(ngram, writer),
				serialize_bag_of_words_feature_group_n_gram_entry(entry, writer),
			)
		})
		.collect::<Vec<_>>();
	let ngrams = writer.write(&ngrams);
	let ngram_types = bag_of_words_cosine_similarity_feature_group
		.ngram_types
		.iter()
		.map(|ngram_type| serialize_ngram_type(ngram_type, writer))
		.collect::<Vec<_>>();
	let ngram_types = writer.write(&ngram_types);
	let strategy = serialize_bag_of_words_feature_group_strategy(
		&bag_of_words_cosine_similarity_feature_group.strategy,
		writer,
	);
	let feature_group = tangram_model::BagOfWordsCosineSimilarityFeatureGroupWriter {
		source_column_name_a,
		source_column_name_b,
		tokenizer,
		strategy,
		ngram_types,
		ngrams,
	};
	writer.write(&feature_group)
}

fn serialize_binary_classification_model(
	binary_classification_model: &BinaryClassificationModel,
	writer: &mut buffalo::Writer,
) -> tangram_model::BinaryClassificationModelWriter {
	match binary_classification_model {
		BinaryClassificationModel::Linear(model) => {
			let linear_binary_classifier =
				serialize_linear_binary_classification_model(&model, writer);
			tangram_model::BinaryClassificationModelWriter::Linear(linear_binary_classifier)
		}
		BinaryClassificationModel::Tree(model) => {
			let tree_binary_classifier = serialize_tree_binary_classification_model(model, writer);
			tangram_model::BinaryClassificationModelWriter::Tree(tree_binary_classifier)
		}
	}
}

fn serialize_linear_binary_classification_model(
	linear_binary_classification_model: &LinearBinaryClassificationModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::LinearBinaryClassifierWriter> {
	let model = linear_binary_classification_model.model.to_writer(writer);
	let train_options =
		serialize_linear_train_options(&linear_binary_classification_model.train_options, writer);
	let feature_groups = linear_binary_classification_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = linear_binary_classification_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let feature_importances = writer.write(
		linear_binary_classification_model
			.feature_importances
			.as_slice(),
	);
	let model = tangram_model::LinearBinaryClassifierWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&model)
}

fn serialize_tree_binary_classification_model(
	tree_binary_classification_model: &TreeBinaryClassificationModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TreeBinaryClassifierWriter> {
	let feature_importances = writer.write(
		tree_binary_classification_model
			.feature_importances
			.as_slice(),
	);
	let train_options =
		serialize_tree_train_options(&tree_binary_classification_model.train_options, writer);
	let feature_groups = tree_binary_classification_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = tree_binary_classification_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let model = tree_binary_classification_model.model.to_writer(writer);
	let model = tangram_model::TreeBinaryClassifierWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&model)
}

fn serialize_binary_classification_metrics_output(
	binary_classification_metrics_output: &tangram_metrics::BinaryClassificationMetricsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BinaryClassificationMetricsWriter> {
	let thresholds = binary_classification_metrics_output
		.thresholds
		.iter()
		.map(|threshold| {
			serialize_binary_classification_metrics_output_for_threshold(threshold, writer)
		})
		.collect::<Vec<_>>();
	let thresholds = writer.write(&thresholds);
	let default_threshold = serialize_binary_classification_metrics_output_for_threshold(
		&binary_classification_metrics_output.thresholds
			[binary_classification_metrics_output.thresholds.len() / 2],
		writer,
	);
	let metrics = tangram_model::BinaryClassificationMetricsWriter {
		auc_roc: binary_classification_metrics_output.auc_roc_approx,
		default_threshold,
		thresholds,
	};
	writer.write(&metrics)
}

fn serialize_binary_classification_metrics_output_for_threshold(
	binary_classification_metrics_output_for_threshold: &tangram_metrics::BinaryClassificationMetricsOutputForThreshold,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::BinaryClassificationMetricsForThresholdWriter> {
	let metrics = tangram_model::BinaryClassificationMetricsForThresholdWriter {
		threshold: binary_classification_metrics_output_for_threshold.threshold,
		true_positives: binary_classification_metrics_output_for_threshold
			.true_positives
			.to_u64()
			.unwrap(),
		false_positives: binary_classification_metrics_output_for_threshold
			.false_positives
			.to_u64()
			.unwrap(),
		true_negatives: binary_classification_metrics_output_for_threshold
			.true_negatives
			.to_u64()
			.unwrap(),
		false_negatives: binary_classification_metrics_output_for_threshold
			.false_negatives
			.to_u64()
			.unwrap(),
		accuracy: binary_classification_metrics_output_for_threshold.accuracy,
		precision: binary_classification_metrics_output_for_threshold.precision,
		recall: binary_classification_metrics_output_for_threshold.recall,
		f1_score: binary_classification_metrics_output_for_threshold.f1_score,
		true_positive_rate: binary_classification_metrics_output_for_threshold.true_positive_rate,
		false_positive_rate: binary_classification_metrics_output_for_threshold.false_positive_rate,
	};
	writer.write(&metrics)
}

fn serialize_binary_classification_comparison_metric(
	binary_classification_comparison_metric: &BinaryClassificationComparisonMetric,
	_writer: &mut buffalo::Writer,
) -> tangram_model::BinaryClassificationComparisonMetricWriter {
	match binary_classification_comparison_metric {
		BinaryClassificationComparisonMetric::AucRoc => {
			tangram_model::BinaryClassificationComparisonMetricWriter::Aucroc
		}
	}
}

fn serialize_multiclass_classification_model(
	multiclass_classification_model: &MulticlassClassificationModel,
	writer: &mut buffalo::Writer,
) -> tangram_model::MulticlassClassificationModelWriter {
	match multiclass_classification_model {
		MulticlassClassificationModel::Linear(model) => {
			let linear_multiclass_classifier =
				serialize_linear_multiclass_classification_model(model, writer);
			tangram_model::MulticlassClassificationModelWriter::Linear(linear_multiclass_classifier)
		}
		MulticlassClassificationModel::Tree(model) => {
			let tree_multiclass_classifier =
				serialize_tree_multiclass_classification_model(model, writer);
			tangram_model::MulticlassClassificationModelWriter::Tree(tree_multiclass_classifier)
		}
	}
}

fn serialize_linear_multiclass_classification_model(
	linear_multiclass_classification_model: &LinearMulticlassClassificationModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::LinearMulticlassClassifierWriter> {
	let feature_importances = writer.write(
		linear_multiclass_classification_model
			.feature_importances
			.as_slice(),
	);
	let train_options = serialize_linear_train_options(
		&linear_multiclass_classification_model.train_options,
		writer,
	);
	let feature_groups = linear_multiclass_classification_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = linear_multiclass_classification_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let model = linear_multiclass_classification_model
		.model
		.to_writer(writer);
	let model = tangram_model::LinearMulticlassClassifierWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&model)
}

fn serialize_tree_multiclass_classification_model(
	tree_multiclass_classification_model: &TreeMulticlassClassificationModel,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::TreeMulticlassClassifierWriter> {
	let feature_importances = writer.write(
		tree_multiclass_classification_model
			.feature_importances
			.as_slice(),
	);
	let train_options =
		serialize_tree_train_options(&tree_multiclass_classification_model.train_options, writer);
	let feature_groups = tree_multiclass_classification_model
		.feature_groups
		.iter()
		.map(|feature_group| serialize_feature_group(feature_group, writer))
		.collect::<Vec<_>>();
	let feature_groups = writer.write(&feature_groups);
	let losses = tree_multiclass_classification_model
		.losses
		.as_ref()
		.map(|losses| writer.write(losses.as_slice()));
	let model = tree_multiclass_classification_model.model.to_writer(writer);
	let model = tangram_model::TreeMulticlassClassifierWriter {
		model,
		train_options,
		feature_groups,
		losses,
		feature_importances,
	};
	writer.write(&model)
}

fn serialize_multiclass_classification_metrics_output(
	multiclass_classification_metrics_output: &tangram_metrics::MulticlassClassificationMetricsOutput,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::MulticlassClassificationMetricsWriter> {
	let class_metrics = multiclass_classification_metrics_output
		.class_metrics
		.iter()
		.map(|class_metric| serialize_class_metrics(&class_metric, writer))
		.collect::<Vec<_>>();
	let class_metrics = writer.write(&class_metrics);
	let metrics = tangram_model::MulticlassClassificationMetricsWriter {
		class_metrics,
		accuracy: multiclass_classification_metrics_output.accuracy,
		precision_unweighted: multiclass_classification_metrics_output.precision_unweighted,
		precision_weighted: multiclass_classification_metrics_output.precision_weighted,
		recall_unweighted: multiclass_classification_metrics_output.recall_weighted,
		recall_weighted: multiclass_classification_metrics_output.recall_weighted,
	};
	writer.write(&metrics)
}

fn serialize_class_metrics(
	class_metrics: &tangram_metrics::ClassMetrics,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<tangram_model::ClassMetricsWriter> {
	let metrics = tangram_model::ClassMetricsWriter {
		true_positives: class_metrics.true_positives.to_u64().unwrap(),
		false_positives: class_metrics.false_positives.to_u64().unwrap(),
		true_negatives: class_metrics.true_negatives.to_u64().unwrap(),
		false_negatives: class_metrics.false_negatives.to_u64().unwrap(),
		accuracy: class_metrics.accuracy,
		precision: class_metrics.precision,
		recall: class_metrics.recall,
		f1_score: class_metrics.f1_score,
	};
	writer.write(&metrics)
}

fn serialize_multiclass_classification_comparison_metric(
	multiclass_classification_comparison_metric: &MulticlassClassificationComparisonMetric,
	_writer: &mut buffalo::Writer,
) -> tangram_model::MulticlassClassificationComparisonMetricWriter {
	match multiclass_classification_comparison_metric {
		MulticlassClassificationComparisonMetric::Accuracy => {
			tangram_model::MulticlassClassificationComparisonMetricWriter::Accuracy
		}
	}
}
