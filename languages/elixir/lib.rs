use anyhow::Result;
use once_cell::sync::OnceCell;
use std::collections::BTreeMap;

erl_nif::init!(
	name: "Elixir.Tangram",
	funcs: [_load_model_from_path, _load_model_from_binary, _model_id, _predict],
	load: load,
);

static MODEL_RESOURCE_TYPE: OnceCell<erl_nif::ResourceType<tangram_core::predict::Model>> =
	OnceCell::new();

fn load<'a>(env: erl_nif::Env<'a>, _load_info: erl_nif::Term<'a>) -> Result<()> {
	let model_resource_type = erl_nif::ResourceType::new(env, "Model")?;
	MODEL_RESOURCE_TYPE
		.set(model_resource_type)
		.ok()
		.expect("failed to set model resource type");
	Ok(())
}

#[erl_nif::nif]
fn _load_model_from_path<'a>(
	env: erl_nif::Env<'a>,
	path: Option<String>,
) -> Result<erl_nif::Term<'a>> {
	let file = std::fs::File::open(path.unwrap())?;
	let bytes = unsafe { memmap::Mmap::map(&file)? };
	let model = tangram_model::from_bytes(&bytes)?;
	let model = tangram_core::predict::Model::from(model);
	let resource_type = MODEL_RESOURCE_TYPE.get().unwrap();
	let model = erl_nif::Resource::new(*resource_type, model);
	let model = erl_nif::ResourceTerm::new(env, model)?;
	Ok(model.term())
}

#[erl_nif::nif]
fn _load_model_from_binary<'a>(
	env: erl_nif::Env<'a>,
	binary: erl_nif::BinaryTerm<'a>,
) -> Result<erl_nif::Term<'a>> {
	let bytes = binary.get()?;
	let model = tangram_model::from_bytes(&bytes)?;
	let model = tangram_core::predict::Model::from(model);
	let resource_type = MODEL_RESOURCE_TYPE.get().unwrap();
	let model = erl_nif::Resource::new(*resource_type, model);
	let model = erl_nif::ResourceTerm::new(env, model)?;
	Ok(model.term())
}

#[erl_nif::nif]
fn _model_id<'a>(env: erl_nif::Env<'a>, model: erl_nif::Term<'a>) -> Result<String> {
	let resource_type = MODEL_RESOURCE_TYPE
		.get()
		.expect("failed to get model resource type");
	let model = model.as_resource(*resource_type)?;
	let model = model.get()?;
	let id = model.id.clone();
	Ok(id)
}

#[erl_nif::nif]
fn _predict<'a>(
	env: erl_nif::Env<'a>,
	model: erl_nif::Term<'a>,
	input: PredictInputSingleOrMultiple,
	options: Option<PredictOptions>,
) -> Result<PredictOutputSingleOrMultiple> {
	let resource_type = MODEL_RESOURCE_TYPE
		.get()
		.expect("failed to get model resource type");
	let model = model.as_resource(*resource_type)?;
	let model = model.get()?;
	let options = options.map(Into::into).unwrap_or_default();
	match input {
		PredictInputSingleOrMultiple::Single(input) => {
			let input = input.into();
			let mut output = tangram_core::predict::predict(&model, &[input], &options);
			let output = output.remove(0);
			let output = output.into();
			let output = PredictOutputSingleOrMultiple::Single(output);
			Ok(output)
		}
		PredictInputSingleOrMultiple::Multiple(input) => {
			let input = input.into_iter().map(Into::into).collect::<Vec<_>>();
			let output = tangram_core::predict::predict(&model, &input, &options);
			let output = output.into_iter().map(Into::into).collect();
			let output = PredictOutputSingleOrMultiple::Multiple(output);
			Ok(output)
		}
	}
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum PredictInputSingleOrMultiple {
	Single(PredictInput),
	Multiple(PredictInputMultiple),
}

impl<'a> erl_nif::FromErlNif<'a> for PredictInputSingleOrMultiple {
	fn from_erl_nif(term: erl_nif::Term) -> erl_nif::Result<Self> {
		serde::Deserialize::deserialize(term)
	}
}

#[derive(serde::Deserialize)]
struct PredictInput(pub BTreeMap<String, PredictInputValue>);

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

#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
struct PredictOptions {
	pub threshold: Option<f32>,
	pub compute_feature_contributions: Option<bool>,
}

impl<'a> erl_nif::FromErlNif<'a> for PredictOptions {
	fn from_erl_nif(term: erl_nif::Term) -> erl_nif::Result<Self> {
		serde::Deserialize::deserialize(term)
	}
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

#[derive(serde::Serialize)]
#[serde(untagged)]
enum PredictOutputSingleOrMultiple {
	Single(PredictOutput),
	Multiple(PredictOutputMultiple),
}

impl<'a> erl_nif::IntoErlNif<'a> for PredictOutputSingleOrMultiple {
	fn into_erl_nif(self, env: erl_nif::Env<'a>) -> erl_nif::Result<erl_nif::Term<'a>> {
		serde::Serialize::serialize(&self, env)
	}
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum PredictOutput {
	Regression(RegressionPredictOutput),
	BinaryClassification(BinaryClassificationPredictOutput),
	MulticlassClassification(MulticlassClassificationPredictOutput),
}

type PredictOutputMultiple = Vec<PredictOutput>;

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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.RegressionPredictOutput")]
struct RegressionPredictOutput {
	pub value: f32,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.BinaryClassificationPredictOutput")]
struct BinaryClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.MulticlassClassificationPredictOutput")]
struct MulticlassClassificationPredictOutput {
	pub class_name: String,
	pub probability: f32,
	pub probabilities: BTreeMap<String, f32>,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.FeatureContributions")]
struct FeatureContributions {
	pub baseline_value: f32,
	pub output_value: f32,
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

#[derive(serde::Serialize)]
enum FeatureContributionEntry {
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.IdentityFeatureContribution")]
struct IdentityFeatureContribution {
	column_name: String,
	feature_contribution_value: f32,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.NormalizedFeatureContribution")]
struct NormalizedFeatureContribution {
	column_name: String,
	feature_contribution_value: f32,
}

impl From<tangram_core::predict::NormalizedFeatureContribution> for NormalizedFeatureContribution {
	fn from(value: tangram_core::predict::NormalizedFeatureContribution) -> Self {
		NormalizedFeatureContribution {
			column_name: value.column_name,
			feature_contribution_value: value.feature_contribution_value,
		}
	}
}

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.OneHotEncodedFeatureContribution")]
struct OneHotEncodedFeatureContribution {
	column_name: String,
	variant: Option<String>,
	feature_value: bool,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.BagOfWordsFeatureContribution")]
struct BagOfWordsFeatureContribution {
	column_name: String,
	ngram: NGram,
	feature_value: f32,
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

#[derive(serde::Serialize)]
#[serde(untagged)]
enum NGram {
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.BagOfWordsCosineSimilarityFeatureContribution")]
struct BagOfWordsCosineSimilarityFeatureContribution {
	column_name_a: String,
	column_name_b: String,
	feature_value: f32,
	feature_contribution_value: f32,
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

#[derive(serde::Serialize)]
#[serde(rename = "Elixir.Tangram.WordEmbeddingFeatureContribution")]
struct WordEmbeddingFeatureContribution {
	column_name: String,
	value_index: usize,
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
