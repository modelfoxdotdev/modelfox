use pyo3::{prelude::*, type_object::PyTypeObject, types::PyType};
use std::collections::BTreeMap;
use tangram_error::err;
use url::Url;

/**
*/
#[pymodule]
fn tangram(py: Python, m: &PyModule) -> PyResult<()> {
	m.add_class::<LoadModelOptions>()?;
	m.add_class::<Model>()?;
	m.add_class::<PredictOptions>()?;
	m.add_class::<RegressionPredictOutput>()?;
	m.add_class::<BinaryClassificationPredictOutput>()?;
	m.add_class::<MulticlassClassificationPredictOutput>()?;
	m.add_class::<FeatureContributions>()?;
	m.add_class::<IdentityFeatureContribution>()?;
	m.add_class::<NormalizedFeatureContribution>()?;
	m.add_class::<OneHotEncodedFeatureContribution>()?;
	m.add_class::<BagOfWordsFeatureContribution>()?;
	m.add_class::<WordEmbeddingFeatureContribution>()?;
	m.add("PredictInput", predict_input(py)?)?;
	m.add("PredictOutput", predict_output(py)?)?;
	m.add("FeatureContributionEntry", feature_contribution_entry(py)?)?;
	m.add("NGram", ngram(py)?)?;
	Ok(())
}

/**
Use this class to load a model, make predictions, and log events to the app.
*/
#[pyclass]
#[derive(Debug)]
struct Model {
	model: tangram_core::predict::Model,
	log_queue: Vec<Event>,
	tangram_url: Url,
}

#[pymethods]
impl Model {
	/**
	Load a model from a `.tangram` file at `path`.

	Args:
		path (str): The path to the `.tangram` file.
		options (Optional[`LoadModelOptions`]): The options to use when loading the model.

	Returns:
		model (`Model`)
	*/
	#[classmethod]
	#[args(options = "None")]
	#[text_signature = "(path, options=None)"]
	fn from_path(
		_cls: &PyType,
		path: String,
		options: Option<LoadModelOptions>,
	) -> PyResult<Model> {
		let file = std::fs::File::open(path)?;
		let bytes = unsafe { memmap::Mmap::map(&file)? };
		let model = tangram_model::from_bytes(&bytes).map_err(TangramError)?;
		let model = tangram_core::predict::Model::from(model);
		let tangram_url = options
			.and_then(|options| options.tangram_url)
			.unwrap_or_else(|| "https://app.tangram.xyz".to_owned());
		let tangram_url = tangram_url
			.parse()
			.map_err(|_| TangramError(err!("Failed to parse tangram_url")))?;
		let model = Model {
			model,
			log_queue: Vec::new(),
			tangram_url,
		};
		Ok(model)
	}

	/**
	Load a model from bytes instead of a file. You should use this only if you already have a `.tangram` loaded into memory. Otherwise, use `Model.from_path`, which is faster because it memory maps the file.

	Args:
		bytes (str): The path to the `.tangram` file.
		options (Optional[`LoadModelOptions`]): The options to use when loading the model.

	Returns:
		model (`Model`)
	*/
	#[classmethod]
	#[args(options = "None")]
	#[text_signature = "(bytes, options=None)"]
	fn from_bytes(
		_cls: &PyType,
		bytes: Vec<u8>,
		options: Option<LoadModelOptions>,
	) -> PyResult<Model> {
		let model = tangram_model::from_bytes(&bytes).map_err(TangramError)?;
		let model = tangram_core::predict::Model::from(model);
		let tangram_url = options
			.and_then(|options| options.tangram_url)
			.unwrap_or_else(|| "https://app.tangram.xyz".to_owned());
		let tangram_url = tangram_url
			.parse()
			.map_err(|_| TangramError(err!("Failed to parse tangram_url")))?;
		let model = Model {
			model,
			log_queue: Vec::new(),
			tangram_url,
		};
		Ok(model)
	}

	/**
	Retrieve the model's id.
		*/
	#[getter]
	fn id(&self) -> String {
		self.model.id.clone()
	}

	/**
	Make a prediction!

	Args:
		input (Union[List[`PredictInput`], `PredictInput`]): A predict input is either a single predict input which is a dict from strings to strings or floats or an array of such dicts. The keys should match the columns in the CSV file you trained your model with.
		options (Optional[`PredictOptions`]): These are the predict options.

	Returns:
		[Union[List[`PredictOutput`], `PredictOutput`]). Return a single output if `input` was a single input, or an array if `input` was an array of `input`s.
	*/
	#[text_signature = "(input, options=None)"]
	fn predict(
		&self,
		input: PredictInputSingleOrMultiple,
		options: Option<&PredictOptions>,
	) -> PredictOutputSingleOrMultiple {
		let model = &self.model;
		let options = options.map(Into::into).unwrap_or_default();
		match input {
			PredictInputSingleOrMultiple::Single(input) => {
				let input = input.into();
				let mut output = tangram_core::predict::predict(model, &[input], &options);
				let output = output.remove(0);
				let output = output.into();
				PredictOutputSingleOrMultiple::Single(output)
			}
			PredictInputSingleOrMultiple::Multiple(input) => {
				let input = input.into_iter().map(Into::into).collect::<Vec<_>>();
				let output = tangram_core::predict::predict(model, &input, &options);
				let output = output.into_iter().map(Into::into).collect();
				PredictOutputSingleOrMultiple::Multiple(output)
			}
		}
	}

	/**
	Send a prediction event to the app. If you want to batch events, you can use `enqueue_log_prediction` instead.

	Args:
		identifier (Union[str, float]): This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
		input (`PredictInput`): A single `PredictInput`.
		output (`PredictOutput`): A single `PredictOutput`.
		options (Optional[`PredictOptions`]): This is the same `PredictOptions` value that you passed to `predict`.
	  */
	#[args(identifier, input, output, options = "None")]
	#[text_signature = "(identifier, input, output, options=None)"]
	fn log_prediction(
		&mut self,
		identifier: NumberOrString,
		input: PredictInput,
		output: PredictOutput,
		options: Option<PredictOptions>,
	) -> PyResult<()> {
		let event = Event::Prediction(self.prediction_event(identifier, input, output, options));
		self.log_event(event)?;
		Ok(())
	}

	/**
	Add a prediction event to the queue. Remember to call `flush_log_queue` at a later point to send the event to the app.
	Args:
		identifier (Union[str, float]): This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
		input (`PredictInput`): A single `PredictInput`.
		output (`PredictOutput`): A single `PredictOutput`.
		options (`PredictOptions`): This is the same `predictOptions` value that you passed to `predict`.
	*/
	#[text_signature = "(identifier, input, output, options=None)"]
	fn enqueue_log_prediction(
		&mut self,
		identifier: NumberOrString,
		input: PredictInput,
		output: PredictOutput,
		options: Option<PredictOptions>,
	) {
		let event = Event::Prediction(self.prediction_event(identifier, input, output, options));
		self.log_queue.push(event);
	}

	/**
	Send a true value event to the app. If you want to batch events, you can use `enqueue_log_true_value` instead.

	Args:
		identifier (Union[str, float]): This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
		true_value (Union[str, float]): This is the true value for the prediction.
	*/
	#[text_signature = "(identifier, true_value)"]
	fn log_true_value(
		&mut self,
		identifier: NumberOrString,
		true_value: NumberOrString,
	) -> PyResult<()> {
		let event = Event::TrueValue(self.true_value_event(identifier, true_value));
		self.log_event(event)?;
		Ok(())
	}

	/**
	Add a true value event to the queue. Remember to call `flush_log_queue` at a later point to send the event to the app.

	Args:
		identifier (Union[str, float]): This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
		true_value (Union[str, float]): This is the true value for the prediction.
	*/
	#[text_signature = "(identifier, true_value)"]
	fn enqueue_log_true_value(&mut self, identifier: NumberOrString, true_value: NumberOrString) {
		let event = Event::TrueValue(self.true_value_event(identifier, true_value));
		self.log_queue.push(event);
	}

	/**
	Send all events in the queue to the app.
	*/
	#[text_signature = "()"]
	fn flush_log_queue(&mut self) -> PyResult<()> {
		let events = self.log_queue.drain(0..self.log_queue.len()).collect();
		self.log_events(events)
	}
}

impl Model {
	fn log_event(&mut self, event: Event) -> PyResult<()> {
		self.log_events(vec![event])
	}

	fn log_events(&mut self, events: Vec<Event>) -> PyResult<()> {
		let mut url = self.tangram_url.clone();
		url.set_path("/track");
		let body = serde_json::to_vec(&events).map_err(|err| TangramError(err.into()))?;
		reqwest::blocking::Client::new()
			.post(url)
			.body(body)
			.send()
			.map_err(|err| TangramError(err.into()))?;
		Ok(())
	}

	fn prediction_event(
		&self,
		identifier: NumberOrString,
		input: PredictInput,
		output: PredictOutput,
		options: Option<PredictOptions>,
	) -> PredictionEvent {
		PredictionEvent {
			date: chrono::Utc::now(),
			identifier,
			input,
			options,
			output,
			model_id: self.id(),
		}
	}

	fn true_value_event(
		&self,
		identifier: NumberOrString,
		true_value: NumberOrString,
	) -> TrueValueEvent {
		TrueValueEvent {
			date: chrono::Utc::now(),
			identifier,
			model_id: self.id(),
			true_value,
		}
	}
}

/**
These are the options passed when loading a model.

Attributes:
	tangram_url (Optional[str]): If you are running the app locally or on your own server, use this field to provide the url to it.
*/
#[pyclass]
#[derive(Clone, Debug)]
struct LoadModelOptions {
	#[pyo3(get, set)]
	tangram_url: Option<String>,
}

#[pymethods]
impl LoadModelOptions {
	#[new]
	fn new(tangram_url: Option<String>) -> LoadModelOptions {
		LoadModelOptions { tangram_url }
	}
}

#[derive(FromPyObject)]
enum PredictInputSingleOrMultiple {
	Single(PredictInput),
	Multiple(PredictInputMultiple),
}

#[derive(Debug, FromPyObject, serde::Serialize)]
struct PredictInput(BTreeMap<String, PredictInputValue>);

type PredictInputMultiple = Vec<PredictInput>;

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

#[derive(Debug, FromPyObject, serde::Serialize)]
#[serde(untagged)]
enum PredictInputValue {
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

/**
These are the options passed to `Model.predict`.

Attributes:
	threshold (Optional[float]): If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.

	compute_feature_contributions (Optional[bool]): Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `featureContributions` field of the predict output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct PredictOptions {
	#[pyo3(get, set)]
	threshold: Option<f32>,
	#[pyo3(get, set)]
	compute_feature_contributions: Option<bool>,
}

#[pymethods]
impl PredictOptions {
	#[new]
	fn new(threshold: Option<f32>, compute_feature_contributions: Option<bool>) -> PredictOptions {
		PredictOptions {
			threshold,
			compute_feature_contributions,
		}
	}
}

impl From<&PredictOptions> for tangram_core::predict::PredictOptions {
	fn from(value: &PredictOptions) -> tangram_core::predict::PredictOptions {
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

enum PredictOutputSingleOrMultiple {
	Single(PredictOutput),
	Multiple(PredictOutputMultiple),
}

impl IntoPy<PyObject> for PredictOutputSingleOrMultiple {
	fn into_py(self, py: Python) -> PyObject {
		match self {
			PredictOutputSingleOrMultiple::Single(s) => s.into_py(py),
			PredictOutputSingleOrMultiple::Multiple(s) => s.into_py(py),
		}
	}
}

#[derive(Debug, serde::Serialize, FromPyObject)]
#[serde(untagged)]
enum PredictOutput {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

type PredictOutputMultiple = Vec<PredictOutput>;

impl IntoPy<PyObject> for PredictOutput {
	fn into_py(self, py: Python) -> PyObject {
		match self {
			PredictOutput::Regression(s) => s.into_py(py),
			PredictOutput::BinaryClassification(s) => s.into_py(py),
			PredictOutput::MulticlassClassification(s) => s.into_py(py),
		}
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

/**
`Model.predict` outputs `RegressionPredictOutput` when the model's task is regression.

Attributes:
	value: This is the predicted value.
	feature_contributions (`FeatureContributions`): If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct RegressionPredictOutput {
	#[pyo3(get)]
	value: f32,
	#[pyo3(get)]
	#[serde(skip_serializing)]
	feature_contributions: Option<FeatureContributions>,
}

impl From<tangram_core::predict::RegressionPredictOutput> for RegressionPredictOutput {
	fn from(value: tangram_core::predict::RegressionPredictOutput) -> Self {
		RegressionPredictOutput {
			value: value.value,
			feature_contributions: value.feature_contributions.map(Into::into),
		}
	}
}

/**
`Model.predict` outputs `BinaryClassificationPredictOutput` when the model's task is binary classification.

Attributes:
	class_name (str): This is the name of the predicted class.
	probability (float): This is the probability the model assigned to the predicted class.
	feature_contributions (`FeatureContributions`): If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct BinaryClassificationPredictOutput {
	#[pyo3(get)]
	class_name: String,
	#[pyo3(get)]
	probability: f32,
	#[pyo3(get)]
	#[serde(skip_serializing)]
	feature_contributions: Option<FeatureContributions>,
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

/**
`Model.predict` outputs `MulticlassClassificationPredictOutput` when the model's task is multiclass classification.

Attributes:
	class_name (str): This is the name of the predicted class.
	probability (float): This is the probability the model assigned to the predicted class.
	probabilities (Dict[str, float]): This value maps from class names to the probability the model assigned to each class.
	feature_contributions (Dict[str, `FeatureContributions`]): If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `FeatureContributions.output_value` is the predicted class.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct MulticlassClassificationPredictOutput {
	#[pyo3(get)]
	class_name: String,
	#[pyo3(get)]
	probability: f32,
	#[pyo3(get)]
	probabilities: BTreeMap<String, f32>,
	#[pyo3(get)]
	#[serde(skip_serializing)]
	feature_contributions: Option<BTreeMap<String, FeatureContributions>>,
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

/**
This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.

Attributes:
	baseline_value (float): This is the value the model would output if all features had baseline values.
	output_value (float): This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.
	entries (List[`FeatureContributionEntry`]): This list will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct FeatureContributions {
	#[pyo3(get)]
	baseline_value: f32,
	#[pyo3(get)]
	output_value: f32,
	#[pyo3(get)]
	entries: Vec<FeatureContributionEntry>,
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
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type")]
enum FeatureContributionEntry {
	#[serde(rename = "identity")]
	Identity(IdentityFeatureContribution),
	#[serde(rename = "normalized")]
	Normalized(NormalizedFeatureContribution),
	#[serde(rename = "one_hot_encoded")]
	OneHotEncoded(OneHotEncodedFeatureContribution),
	#[serde(rename = "bag_of_words")]
	BagOfWords(BagOfWordsFeatureContribution),
	#[serde(rename = "word_embedding")]
	WordEmbedding(WordEmbeddingFeatureContribution),
}

impl IntoPy<PyObject> for FeatureContributionEntry {
	fn into_py(self, py: Python) -> PyObject {
		match self {
			FeatureContributionEntry::Identity(s) => s.into_py(py),
			FeatureContributionEntry::Normalized(s) => s.into_py(py),
			FeatureContributionEntry::OneHotEncoded(s) => s.into_py(py),
			FeatureContributionEntry::BagOfWords(s) => s.into_py(py),
			FeatureContributionEntry::WordEmbedding(s) => s.into_py(py),
		}
	}
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
			tangram_core::predict::FeatureContributionEntry::WordEmbedding(value) => {
				FeatureContributionEntry::WordEmbedding(value.into())
			}
		}
	}
}

/**
This describes the contribution of a feature from an identity feature group.

Attributes:
	column_name (str): This is the name of the source column for the feature group.
	feature_value (float): This is the value of the feature.
	feature_contribution_value (float): This is the amount that the feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct IdentityFeatureContribution {
	#[pyo3(get)]
	column_name: String,
	#[pyo3(get)]
	feature_contribution_value: f32,
	#[pyo3(get)]
	feature_value: f32,
}

impl From<tangram_core::predict::IdentityFeatureContribution> for IdentityFeatureContribution {
	fn from(value: tangram_core::predict::IdentityFeatureContribution) -> Self {
		IdentityFeatureContribution {
			column_name: value.column_name,
			feature_contribution_value: value.feature_contribution_value,
			feature_value: value.feature_value,
		}
	}
}

/**
This describes the contribution of a feature from a normalized feature group.

Attributes:
	column_name (str): This is the name of the source column for the feature group.
	feature_value (float): This is the value of the feature.
	feature_contribution_value (float): This is the amount that the feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct NormalizedFeatureContribution {
	#[pyo3(get)]
	column_name: String,
	#[pyo3(get)]
	feature_value: f32,
	#[pyo3(get)]
	feature_contribution_value: f32,
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

/**
This describes the contribution of a feature from a one hot encoded feature group.

Attributes:
	column_name (str): This is the name of the source column for the feature group.
	variant (str): This is the enum variant the feature indicates the presence of.
	feature_value (float): This is the value of the feature.
	feature_contribution_value (float): This is the amount that the feature contributed to the output.b
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct OneHotEncodedFeatureContribution {
	#[pyo3(get)]
	column_name: String,
	#[pyo3(get)]
	variant: Option<String>,
	#[pyo3(get)]
	feature_value: bool,
	#[pyo3(get)]
	feature_contribution_value: f32,
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

/**
This describes the contribution of a feature from a bag of words feature group.

Attributes:
	column_name (str): This is the name of the source column for the feature group.
	ngram (`NGram`): This is the ngram for the feature.
	feature_value (float): This is the value of the feature..
	feature_contribution_value (float): This is the amount that the feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct BagOfWordsFeatureContribution {
	#[pyo3(get)]
	column_name: String,
	#[pyo3(get)]
	ngram: NGram,
	#[pyo3(get)]
	feature_value: bool,
	#[pyo3(get)]
	feature_contribution_value: f32,
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

#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
enum NGram {
	Unigram(String),
	Bigram(String, String),
}

impl IntoPy<PyObject> for NGram {
	fn into_py(self, py: Python) -> PyObject {
		match self {
			NGram::Unigram(token) => token.into_py(py),
			NGram::Bigram(token_a, token_b) => vec![token_a, token_b].into_py(py),
		}
	}
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

/**
This describes the contribution of a feature from a word embedding feature group.

Attributes:
	column_name (str): This is the name of the source column for the feature group.
	value_index (int): This is the index of the feature in the word embedding.
	feature_contribution_value (float): This is the amount that the feature contributed to the output.
*/
#[pyclass]
#[derive(Clone, Debug, serde::Serialize)]
struct WordEmbeddingFeatureContribution {
	#[pyo3(get)]
	column_name: String,
	#[pyo3(get)]
	value_index: usize,
	#[pyo3(get)]
	feature_contribution_value: f32,
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

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum Event {
	#[serde(rename = "prediction")]
	Prediction(PredictionEvent),
	#[serde(rename = "true_value")]
	TrueValue(TrueValueEvent),
}

#[pyclass]
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

#[derive(Debug, serde::Serialize, FromPyObject)]
#[serde(untagged)]
enum NumberOrString {
	Number(f64),
	String(String),
}

fn predict_input(py: Python) -> PyResult<PyObject> {
	let typing = py.import("typing")?;
	let py_dict = typing.getattr("Dict")?;
	let py_str = py.eval("str", None, None)?;
	let py_any = typing.getattr("Any")?;
	let predict_input = py_dict.get_item((py_str, py_any))?;
	Ok(predict_input.into())
}

fn predict_output(py: Python) -> PyResult<PyObject> {
	let typing = py.import("typing")?;
	let py_union = typing.getattr("Union")?;
	let predict_output = py_union.get_item((
		RegressionPredictOutput::type_object(py),
		BinaryClassificationPredictOutput::type_object(py),
		MulticlassClassificationPredictOutput::type_object(py),
	))?;
	Ok(predict_output.into())
}

fn feature_contribution_entry(py: Python) -> PyResult<PyObject> {
	let typing = py.import("typing")?;
	let py_union = typing.getattr("Union")?;
	let feature_contribution_entry = py_union.get_item((
		IdentityFeatureContribution::type_object(py),
		NormalizedFeatureContribution::type_object(py),
		OneHotEncodedFeatureContribution::type_object(py),
		BagOfWordsFeatureContribution::type_object(py),
		WordEmbeddingFeatureContribution::type_object(py),
	))?;
	Ok(feature_contribution_entry.into())
}

fn ngram(py: Python) -> PyResult<PyObject> {
	let typing = py.import("typing")?;
	let py_union = typing.getattr("Union")?;
	let py_tuple = typing.getattr("Tuple")?;
	let py_str = py.eval("str", None, None)?;
	let tuple = py_tuple.get_item((py_str, py_str))?;
	let ngram = py_union.get_item((tuple, py_str))?;
	Ok(ngram.into())
}

macro_rules! repr {
	($ty:ty) => {
		#[pyproto]
		impl pyo3::PyObjectProtocol for $ty {
			fn __repr__(&self) -> PyResult<String> {
				Ok(format!("{:?}", self))
			}
		}
	};
}

repr!(RegressionPredictOutput);
repr!(BinaryClassificationPredictOutput);
repr!(MulticlassClassificationPredictOutput);

struct TangramError(tangram_error::Error);

impl std::fmt::Display for TangramError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<TangramError> for PyErr {
	fn from(error: TangramError) -> PyErr {
		PyErr::new::<pyo3::exceptions::PyTypeError, _>(error.to_string())
	}
}
