/*!
The Tangram crate makes it easy to make predictions with your Tangram machine learning model from Rust.

## Usage

```toml
[dependencies]
tangram = { version = "*" }
```

```rust
let model = tangram::Model::<Input, Output>::from_path("examples/heart-disease.tangram");

let input = tangram::predict_input! {
  "age": 63,
  "gender": "male",
  // ...
};

let output = model.predict_one(input, None);
```

For more information, [read the docs](https://www.tangram.xyz/docs).
*/

use std::path::Path;
use std::{collections::BTreeMap, marker::PhantomData};
use tangram_error::Result;
use url::Url;

pub use tangram_macro::{predict_input, PredictInput, PredictInputValue};

/// Use this struct to load a model, make predictions, and log events to the app.
pub struct Model<Input = PredictInput, Output = PredictOutput>
where
	Input: Into<PredictInput>,
	Output: From<PredictOutput> + Into<PredictOutput>,
{
	model: tangram_core::predict::Model,
	log_queue: Vec<Event>,
	tangram_url: Url,
	input_marker: PhantomData<Input>,
	output_marker: PhantomData<Output>,
}

/// These are the options passed when loading a [`Model`].
pub struct LoadModelOptions {
	/// If you are running the app locally or on your own server, use this field to provide a url that points to it. If not specified, the default value is https://app.tangram.xyz.
	pub tangram_url: Option<Url>,
}

/// This is the input type of [`Model::predict`]. A predict input is a map whose keys are the same as the column names in the CSV the model was trained with, and whose values match the type for each column.
#[derive(Clone, Debug, serde::Serialize)]
pub struct PredictInput(pub BTreeMap<String, PredictInputValue>);

impl From<PredictInput> for tangram_core::predict::PredictInput {
	fn from(value: PredictInput) -> tangram_core::predict::PredictInput {
		tangram_core::predict::PredictInput(
			value
				.0
				.into_iter()
				.map(|(key, value)| (key, value.into()))
				.collect(),
		)
	}
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum PredictInputValue {
	Number(f64),
	String(String),
}

impl From<PredictInputValue> for tangram_core::predict::PredictInputValue {
	fn from(value: PredictInputValue) -> tangram_core::predict::PredictInputValue {
		match value {
			PredictInputValue::Number(value) => {
				tangram_core::predict::PredictInputValue::Number(value)
			}
			PredictInputValue::String(value) => {
				tangram_core::predict::PredictInputValue::String(value)
			}
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
		PredictInputValue::Number(value as f64)
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

/// These are the options passed to [`Model::predict`].
#[derive(Clone, Debug, serde::Serialize)]
pub struct PredictOptions {
	/// If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.
	pub threshold: Option<f32>,
	/// Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `feature_contributions` field of the predict output.
	pub compute_feature_contributions: Option<bool>,
}

impl From<PredictOptions> for tangram_core::predict::PredictOptions {
	fn from(value: PredictOptions) -> tangram_core::predict::PredictOptions {
		let mut options = tangram_core::predict::PredictOptions::default();
		if let Some(threshold) = value.threshold {
			options.threshold = threshold;
		}
		if let Some(compute_feature_contributions) = value.compute_feature_contributions {
			options.compute_feature_contributions = compute_feature_contributions;
		}
		options
	}
}

/// This is the output of [`Model::predict`].
#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum PredictOutput {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

impl From<RegressionPredictOutput> for PredictOutput {
	fn from(value: RegressionPredictOutput) -> Self {
		PredictOutput::Regression(value)
	}
}

impl From<BinaryClassificationPredictOutput> for PredictOutput {
	fn from(value: BinaryClassificationPredictOutput) -> Self {
		PredictOutput::BinaryClassification(value)
	}
}

impl From<MulticlassClassificationPredictOutput> for PredictOutput {
	fn from(value: MulticlassClassificationPredictOutput) -> Self {
		PredictOutput::MulticlassClassification(value)
	}
}

impl From<tangram_core::predict::PredictOutput> for PredictOutput {
	fn from(value: tangram_core::predict::PredictOutput) -> Self {
		match value {
			tangram_core::predict::PredictOutput::Regression(value) => {
				PredictOutput::Regression(value.into())
			}
			tangram_core::predict::PredictOutput::BinaryClassification(value) => {
				PredictOutput::BinaryClassification(value.into())
			}
			tangram_core::predict::PredictOutput::MulticlassClassification(value) => {
				PredictOutput::MulticlassClassification(value.into())
			}
		}
	}
}

/// This is the output of calling [`Model::predict`] on a `Model` whose task is regression.
#[derive(Debug, serde::Serialize)]
pub struct RegressionPredictOutput {
	/// This is the predicted value.
	pub value: f32,
	/// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	pub feature_contributions: Option<FeatureContributions>,
}

impl From<tangram_core::predict::RegressionPredictOutput> for RegressionPredictOutput {
	fn from(value: tangram_core::predict::RegressionPredictOutput) -> Self {
		RegressionPredictOutput {
			value: value.value,
			feature_contributions: value.feature_contributions.map(Into::into),
		}
	}
}

impl From<PredictOutput> for RegressionPredictOutput {
	fn from(value: PredictOutput) -> Self {
		match value {
			PredictOutput::Regression(value) => value,
			_ => panic!("expected regression predict output"),
		}
	}
}

impl From<PredictOutput> for BinaryClassificationPredictOutput {
	fn from(value: PredictOutput) -> Self {
		match value {
			PredictOutput::BinaryClassification(value) => value,
			_ => panic!("expected binary classification predict output"),
		}
	}
}

impl From<PredictOutput> for MulticlassClassificationPredictOutput {
	fn from(value: PredictOutput) -> Self {
		match value {
			PredictOutput::MulticlassClassification(value) => value,
			_ => panic!("expected multiclass classification predict output"),
		}
	}
}

/// This is the output of calling [`Model::predict`] on a `Model` whose task is binary classification.
#[derive(Debug, serde::Serialize)]
pub struct BinaryClassificationPredictOutput {
	/// This is the name of the predicted class.
	pub class_name: String,
	/// This is the probability the model assigned to the predicted class.
	pub probability: f32,
	/// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	pub feature_contributions: Option<FeatureContributions>,
}

impl From<tangram_core::predict::BinaryClassificationPredictOutput>
	for BinaryClassificationPredictOutput
{
	fn from(value: tangram_core::predict::BinaryClassificationPredictOutput) -> Self {
		BinaryClassificationPredictOutput {
			class_name: value.class_name,
			probability: value.probability,
			feature_contributions: value.feature_contributions.map(Into::into),
		}
	}
}

/// This is the output of calling [`Model::predict`] on a `Model` whose task is multiclass classification.
#[derive(Debug, serde::Serialize)]
pub struct MulticlassClassificationPredictOutput {
	/// This is the name of the predicted class.
	pub class_name: String,
	/// This is the probability the model assigned to the predicted class.
	pub probability: f32,
	/// This value maps from class names to the probability the model assigned to each class.
	pub probabilities: BTreeMap<String, f32>,
	/// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `output_value` is the predicted class.
	pub feature_contributions: Option<BTreeMap<String, FeatureContributions>>,
}

impl From<tangram_core::predict::MulticlassClassificationPredictOutput>
	for MulticlassClassificationPredictOutput
{
	fn from(value: tangram_core::predict::MulticlassClassificationPredictOutput) -> Self {
		MulticlassClassificationPredictOutput {
			class_name: value.class_name,
			probability: value.probability,
			probabilities: value.probabilities,
			feature_contributions: value.feature_contributions.map(|feature_contributions| {
				feature_contributions
					.into_iter()
					.map(|(key, value)| (key, value.into()))
					.collect()
			}),
		}
	}
}

/// This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.
#[derive(Debug, serde::Serialize)]
pub struct FeatureContributions {
	/// This is the value the model would output if all features had baseline values.
	pub baseline_value: f32,
	/// This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.
	pub output_value: f32,
	/// This vec will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
	pub entries: Vec<FeatureContributionEntry>,
}

impl From<tangram_core::predict::FeatureContributions> for FeatureContributions {
	fn from(value: tangram_core::predict::FeatureContributions) -> Self {
		FeatureContributions {
			baseline_value: value.baseline_value,
			output_value: value.output_value,
			entries: value.entries.into_iter().map(Into::into).collect(),
		}
	}
}

/// This identifies the type of a feature contribution.
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

impl From<tangram_core::predict::FeatureContributionEntry> for FeatureContributionEntry {
	fn from(value: tangram_core::predict::FeatureContributionEntry) -> Self {
		match value {
			tangram_core::predict::FeatureContributionEntry::Identity(value) => {
				FeatureContributionEntry::Identity(value.into())
			}
			tangram_core::predict::FeatureContributionEntry::Normalized(value) => {
				FeatureContributionEntry::Normalized(value.into())
			}
			tangram_core::predict::FeatureContributionEntry::OneHotEncoded(value) => {
				FeatureContributionEntry::OneHotEncoded(value.into())
			}
			tangram_core::predict::FeatureContributionEntry::BagOfWords(value) => {
				FeatureContributionEntry::BagOfWords(value.into())
			}
			tangram_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(value) => {
				FeatureContributionEntry::BagOfWordsCosineSimilarity(value.into())
			}
			tangram_core::predict::FeatureContributionEntry::WordEmbedding(value) => {
				FeatureContributionEntry::WordEmbedding(value.into())
			}
		}
	}
}

/// This describes the contribution of a feature from an identity feature group.
#[derive(Debug, serde::Serialize)]
pub struct IdentityFeatureContribution {
	/// This is the name of the source column for the identity feature group.
	pub column_name: String,
	/// This is the value of the feature.
	pub feature_value: f32,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::IdentityFeatureContribution> for IdentityFeatureContribution {
	fn from(value: tangram_core::predict::IdentityFeatureContribution) -> Self {
		IdentityFeatureContribution {
			column_name: value.column_name,
			feature_value: value.feature_value,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

/// This describes the contribution of a feature from a normalized feature group.
#[derive(Debug, serde::Serialize)]
pub struct NormalizedFeatureContribution {
	/// This is the name of the source column for the feature group.
	pub column_name: String,
	/// This is the value of the feature.
	pub feature_value: f32,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::NormalizedFeatureContribution> for NormalizedFeatureContribution {
	fn from(value: tangram_core::predict::NormalizedFeatureContribution) -> Self {
		NormalizedFeatureContribution {
			column_name: value.column_name,
			feature_value: value.feature_value,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

#[derive(Debug, serde::Serialize)]
pub struct OneHotEncodedFeatureContribution {
	/// This is the name of the source column for the feature group.
	pub column_name: String,
	/// This is the enum variant the feature indicates the presence of.
	pub variant: Option<String>,
	/// This is the value of the feature.
	pub feature_value: bool,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::OneHotEncodedFeatureContribution>
	for OneHotEncodedFeatureContribution
{
	fn from(value: tangram_core::predict::OneHotEncodedFeatureContribution) -> Self {
		OneHotEncodedFeatureContribution {
			column_name: value.column_name,
			variant: value.variant,
			feature_value: value.feature_value,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

/// This describes the contribution of a feature from a bag of words feature group.
#[derive(Debug, serde::Serialize)]
pub struct BagOfWordsFeatureContribution {
	/// This is the name of the source column for the feature group.
	pub column_name: String,
	/// This is the ngram for the feature.
	pub ngram: NGram,
	/// This is the value of the feature.
	pub feature_value: f32,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::BagOfWordsFeatureContribution> for BagOfWordsFeatureContribution {
	fn from(value: tangram_core::predict::BagOfWordsFeatureContribution) -> Self {
		BagOfWordsFeatureContribution {
			column_name: value.column_name,
			ngram: value.ngram.into(),
			feature_value: value.feature_value,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

/// This is a sequence of `n` tokens. Tangram currently supports unigrams and bigrams.
#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum NGram {
	Unigram(String),
	Bigram(String, String),
}

impl From<tangram_core::predict::NGram> for NGram {
	fn from(value: tangram_core::predict::NGram) -> Self {
		match value {
			tangram_core::predict::NGram::Unigram(token) => NGram::Unigram(token),
			tangram_core::predict::NGram::Bigram(token_a, token_b) => {
				NGram::Bigram(token_a, token_b)
			}
		}
	}
}

/// This describes the contribution of a feature from a bag of words cosine similarity feature group.
#[derive(Debug, serde::Serialize)]
pub struct BagOfWordsCosineSimilarityFeatureContribution {
	/// This is the name of the first source column for the feature group.
	pub column_name_a: String,
	/// This is the name of the second source column for the feature group.
	pub column_name_b: String,
	/// This is the value of the feature.
	pub feature_value: f32,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::BagOfWordsCosineSimilarityFeatureContribution>
	for BagOfWordsCosineSimilarityFeatureContribution
{
	fn from(value: tangram_core::predict::BagOfWordsCosineSimilarityFeatureContribution) -> Self {
		BagOfWordsCosineSimilarityFeatureContribution {
			column_name_a: value.column_name_a,
			column_name_b: value.column_name_b,
			feature_value: value.feature_value,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

/// This describes the contribution of a feature from a word vector feature group.
#[derive(Debug, serde::Serialize)]
pub struct WordEmbeddingFeatureContribution {
	/// This is the name of the source column for the feature group.
	pub column_name: String,
	/// This is the index of the feature in the word embedding.
	pub value_index: usize,
	/// This is the amount that the feature contributed to the output.
	pub feature_contribution_value: f32,
}

impl From<tangram_core::predict::WordEmbeddingFeatureContribution>
	for WordEmbeddingFeatureContribution
{
	fn from(value: tangram_core::predict::WordEmbeddingFeatureContribution) -> Self {
		WordEmbeddingFeatureContribution {
			column_name: value.column_name,
			value_index: value.value_index,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

/// This is the type of the argument to [`Model::log_prediction`] and [`Model::enqueue_log_prediction`] which specifies the details of the prediction to log.
#[derive(Debug)]
pub struct LogPredictionArgs<Input, Output>
where
	Input: Into<PredictInput>,
	Output: From<PredictOutput>,
{
	/// This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
	pub identifier: NumberOrString,
	/// This is the same [`struct@PredictInput`] value that you passed to [`Model::predict`].
	pub input: Input,
	/// This is the same `PredictOptions` value that you passed to [`Model::predict`].
	pub options: Option<PredictOptions>,
	/// This is the output returned by [`Model::predict`].
	pub output: Output,
}

/// This is the type of the argument to [`Model::log_true_value`] and [`Model::enqueue_log_true_value`] which specifies the details of the true value to log.
#[derive(Debug)]
pub struct LogTrueValueArgs {
	/// This is a unique identifier for the true value, which will associate it with a prediction event and allow you to look it up in the app.
	pub identifier: NumberOrString,
	/// This is the true value for the prediction.
	pub true_value: NumberOrString,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum Event {
	#[serde(rename = "prediction")]
	Prediction(PredictionEvent),
	#[serde(rename = "true_value")]
	TrueValue(TrueValueEvent),
}

#[derive(Debug, serde::Serialize)]
struct PredictionEvent {
	date: chrono::DateTime<chrono::Utc>,
	identifier: NumberOrString,
	input: PredictInput,
	options: Option<PredictOptions>,
	output: PredictOutput,
	model_id: String,
}

#[derive(Debug, serde::Serialize)]
struct TrueValueEvent {
	date: chrono::DateTime<chrono::Utc>,
	identifier: NumberOrString,
	model_id: String,
	true_value: NumberOrString,
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum NumberOrString {
	Number(f64),
	String(String),
}

impl From<f64> for NumberOrString {
	fn from(value: f64) -> Self {
		NumberOrString::Number(value)
	}
}

impl From<String> for NumberOrString {
	fn from(value: String) -> Self {
		NumberOrString::String(value)
	}
}

impl From<&str> for NumberOrString {
	fn from(value: &str) -> Self {
		NumberOrString::String(value.to_owned())
	}
}

/// Use this class to load a model, make predictions, and log events to the app.
impl<Input, Output> Model<Input, Output>
where
	Input: Into<PredictInput>,
	Output: From<PredictOutput> + Into<PredictOutput>,
{
	/// Load a model from the `.tangram` file at `path`.
	pub fn from_path(
		path: impl AsRef<Path>,
		options: Option<LoadModelOptions>,
	) -> Result<Model<Input, Output>> {
		let file = std::fs::File::open(path)?;
		let bytes = unsafe { memmap::Mmap::map(&file)? };
		Model::from_bytes(&bytes, options)
	}

	/// Load a model from a byte slice. You should use this only if you already have a `.tangram` loaded into memory. Otherwise, use [`Model::from_path`], which is faster because it memory maps the file.
	pub fn from_bytes(
		bytes: &[u8],
		options: Option<LoadModelOptions>,
	) -> Result<Model<Input, Output>> {
		let model = tangram_model::from_bytes(&bytes)?;
		let model = tangram_core::predict::Model::from(model);
		let tangram_url = options
			.and_then(|options| options.tangram_url)
			.unwrap_or_else(|| "https://app.tangram.xyz".parse().unwrap());
		Ok(Model {
			model,
			log_queue: Vec::new(),
			tangram_url,
			input_marker: PhantomData,
			output_marker: PhantomData,
		})
	}

	/// Retrieve the model's id.
	pub fn id(&self) -> &str {
		self.model.id.as_str()
	}

	/// Make a prediction with a single input.
	pub fn predict_one(&self, input: Input, options: Option<PredictOptions>) -> Output {
		let model = &self.model;
		let options = options.map(Into::into).unwrap_or_default();
		let output = tangram_core::predict::predict(model, &[input.into().into()], &options);
		let output: PredictOutput = output.into_iter().next().unwrap().into();
		output.into()
	}

	/// Make a prediction with multiple inputs.
	pub fn predict(&self, input: Vec<Input>, options: Option<PredictOptions>) -> Vec<Output> {
		let model = &self.model;
		let options = options.map(Into::into).unwrap_or_default();
		let input = input
			.into_iter()
			.map(Into::into)
			.map(Into::into)
			.collect::<Vec<_>>();
		let output = tangram_core::predict::predict(model, &input, &options);
		output
			.into_iter()
			.map(|output| -> PredictOutput { output.into() })
			.map(Into::into)
			.collect()
	}

	/// Send a prediction event to the app. If you want to batch events, you can use [`Model::enqueue_log_true_value`] instead.
	#[cfg(not(feature = "tokio"))]
	pub fn log_prediction(&mut self, args: LogPredictionArgs<Input, Output>) -> Result<()> {
		let event = Event::Prediction(self.prediction_event(args));
		self.log_event(event)?;
		Ok(())
	}

	/// Send a prediction event to the app. If you want to batch events, you can use [`Model::enqueue_log_true_value`] instead.
	#[cfg(feature = "tokio")]
	pub async fn log_prediction(&mut self, args: LogPredictionArgs<Input, Output>) -> Result<()> {
		let event = Event::Prediction(self.prediction_event(args));
		self.log_event(event).await?;
		Ok(())
	}

	/// Send a true value event to the app. If you want to batch events, you can use [`Model::enqueue_log_true_value`] instead.
	#[cfg(not(feature = "tokio"))]
	pub fn log_true_value(&mut self, args: LogTrueValueArgs) -> Result<()> {
		let event = Event::TrueValue(self.true_value_event(args));
		self.log_event(event)?;
		Ok(())
	}

	/// Send a true value event to the app. If you want to batch events, you can use [`Model::enqueue_log_true_value`] instead.
	#[cfg(feature = "tokio")]
	pub async fn log_true_value(&mut self, args: LogTrueValueArgs) -> Result<()> {
		let event = Event::TrueValue(self.true_value_event(args));
		self.log_event(event).await?;
		Ok(())
	}

	/// Add a prediction event to the queue. Remember to call [`Model::flush_log_queue`] at a later point to send the event to the app.
	pub fn enqueue_log_prediction(&mut self, args: LogPredictionArgs<Input, Output>) {
		let event = Event::Prediction(self.prediction_event(args));
		self.log_queue.push(event);
	}

	/// Add a true value event to the queue. Remember to call [`Model::flush_log_queue`] at a later point to send the event to the app.
	pub fn enqueue_log_true_value(&mut self, args: LogTrueValueArgs) {
		let event = Event::TrueValue(self.true_value_event(args));
		self.log_queue.push(event);
	}

	/// Send all events in the queue to the app.
	#[cfg(not(feature = "tokio"))]
	pub fn flush_log_queue(&mut self) -> Result<()> {
		let events = self.log_queue.drain(0..self.log_queue.len()).collect();
		self.log_events(events)
	}

	/// Send all events in the queue to the app.
	#[cfg(feature = "tokio")]
	pub async fn flush_log_queue(&mut self) -> Result<()> {
		let events = self.log_queue.drain(0..self.log_queue.len()).collect();
		self.log_events(events)
	}

	#[cfg(not(feature = "tokio"))]
	fn log_event(&mut self, event: Event) -> Result<()> {
		self.log_events(vec![event])
	}

	#[cfg(feature = "tokio")]
	fn log_event(&mut self, event: Event) -> Result<()> {
		self.log_events(vec![event])
	}

	#[cfg(not(feature = "tokio"))]
	fn log_events(&mut self, events: Vec<Event>) -> Result<()> {
		let mut url = self.tangram_url.clone();
		url.set_path("/track");
		let body = serde_json::to_vec(&events)?;
		reqwest::blocking::Client::new()
			.post(url)
			.body(body)
			.send()?;
		Ok(())
	}

	#[cfg(feature = "tokio")]
	async fn log_events(&mut self, events: Vec<Event>) -> Result<()> {
		let mut url = self.tangram_url.clone();
		url.set_path("/track");
		let body = serde_json::to_vec(&events)?;
		reqwest::Client::new().post(url).body(body).send().await?;
		Ok(())
	}

	fn prediction_event(&self, args: LogPredictionArgs<Input, Output>) -> PredictionEvent {
		PredictionEvent {
			date: chrono::Utc::now(),
			identifier: args.identifier,
			input: args.input.into(),
			options: args.options,
			output: args.output.into(),
			model_id: self.id().to_owned(),
		}
	}

	fn true_value_event(&self, args: LogTrueValueArgs) -> TrueValueEvent {
		TrueValueEvent {
			date: chrono::Utc::now(),
			identifier: args.identifier,
			model_id: self.id().to_owned(),
			true_value: args.true_value,
		}
	}
}
