use crate::{NGram, NGramType, Tokenizer};

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum FeatureGroup {
	#[tangram_serialize(id = 0)]
	Identity(IdentityFeatureGroup),
	#[tangram_serialize(id = 1)]
	Normalized(NormalizedFeatureGroup),
	#[tangram_serialize(id = 2)]
	OneHotEncoded(OneHotEncodedFeatureGroup),
	#[tangram_serialize(id = 3)]
	BagOfWords(BagOfWordsFeatureGroup),
	#[tangram_serialize(id = 4)]
	WordEmbedding(WordEmbeddingFeatureGroup),
	#[tangram_serialize(id = 5)]
	BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureGroup),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct IdentityFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name: String,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct NormalizedFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub mean: f32,
	#[tangram_serialize(id = 2, required)]
	pub variance: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct OneHotEncodedFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub variants: Vec<String>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BagOfWordsFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[tangram_serialize(id = 2, required)]
	pub strategy: BagOfWordsFeatureGroupStrategy,
	#[tangram_serialize(id = 3, required)]
	pub ngram_types: Vec<NGramType>,
	#[tangram_serialize(id = 4, required)]
	pub ngrams: Vec<(NGram, BagOfWordsFeatureGroupNGramEntry)>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BagOfWordsCosineSimilarityFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name_a: String,
	#[tangram_serialize(id = 1, required)]
	pub source_column_name_b: String,
	#[tangram_serialize(id = 2, required)]
	pub tokenizer: Tokenizer,
	#[tangram_serialize(id = 3, required)]
	pub strategy: BagOfWordsFeatureGroupStrategy,
	#[tangram_serialize(id = 4, required)]
	pub ngram_types: Vec<NGramType>,
	#[tangram_serialize(id = 5, required)]
	pub ngrams: Vec<(NGram, BagOfWordsFeatureGroupNGramEntry)>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum BagOfWordsFeatureGroupStrategy {
	/// The feature values will be 1 if the token is present in the source column value and 0 otherwise.
	#[tangram_serialize(id = 0)]
	Present,
	/// The feature values will be equal to the number of occurrences of the token in the source column value.
	#[tangram_serialize(id = 1)]
	Count,
	/// The feature values will be equal to the number of occurrences of the token in the source column value multiplied by the token's IDF.
	#[tangram_serialize(id = 2)]
	TfIdf,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct BagOfWordsFeatureGroupNGramEntry {
	#[tangram_serialize(id = 0, required)]
	pub idf: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct WordEmbeddingFeatureGroup {
	#[tangram_serialize(id = 0, required)]
	pub source_column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[tangram_serialize(id = 2, required)]
	pub model: WordEmbeddingModel,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct WordEmbeddingModel {
	/// This is the number of values for each word.
	#[tangram_serialize(id = 0, required)]
	pub size: u64,
	/// This is a map from words to row indexes in `values`.
	#[tangram_serialize(id = 1, required)]
	pub words: Vec<(String, u64)>,
	/// This holds the word embedding values, stored as a 2d array of shape (n_words, dimension) in row major order.
	#[tangram_serialize(id = 2, required)]
	pub values: Vec<f32>,
}
