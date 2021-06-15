use crate::{NGram, NGramType, Tokenizer};

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum FeatureGroup {
	#[buffalo(id = 0)]
	Identity(IdentityFeatureGroup),
	#[buffalo(id = 1)]
	Normalized(NormalizedFeatureGroup),
	#[buffalo(id = 2)]
	OneHotEncoded(OneHotEncodedFeatureGroup),
	#[buffalo(id = 3)]
	BagOfWords(BagOfWordsFeatureGroup),
	#[buffalo(id = 4)]
	WordEmbedding(WordEmbeddingFeatureGroup),
	#[buffalo(id = 5)]
	BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureGroup),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct IdentityFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name: String,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct NormalizedFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name: String,
	#[buffalo(id = 1, required)]
	pub mean: f32,
	#[buffalo(id = 2, required)]
	pub variance: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct OneHotEncodedFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name: String,
	#[buffalo(id = 1, required)]
	pub variants: Vec<String>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BagOfWordsFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name: String,
	#[buffalo(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[buffalo(id = 2, required)]
	pub strategy: BagOfWordsFeatureGroupStrategy,
	#[buffalo(id = 3, required)]
	pub ngram_types: Vec<NGramType>,
	#[buffalo(id = 4, required)]
	pub ngrams: Vec<(NGram, BagOfWordsFeatureGroupNGramEntry)>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BagOfWordsCosineSimilarityFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name_a: String,
	#[buffalo(id = 1, required)]
	pub source_column_name_b: String,
	#[buffalo(id = 2, required)]
	pub tokenizer: Tokenizer,
	#[buffalo(id = 3, required)]
	pub strategy: BagOfWordsFeatureGroupStrategy,
	#[buffalo(id = 4, required)]
	pub ngram_types: Vec<NGramType>,
	#[buffalo(id = 5, required)]
	pub ngrams: Vec<(NGram, BagOfWordsFeatureGroupNGramEntry)>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum BagOfWordsFeatureGroupStrategy {
	/// The feature values will be 1 if the token is present in the source column value and 0 otherwise.
	#[buffalo(id = 0)]
	Present,
	/// The feature values will be equal to the number of occurrences of the token in the source column value.
	#[buffalo(id = 1)]
	Count,
	/// The feature values will be equal to the number of occurrences of the token in the source column value multiplied by the token's IDF.
	#[buffalo(id = 2)]
	TfIdf,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BagOfWordsFeatureGroupNGramEntry {
	#[buffalo(id = 0, required)]
	pub idf: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct WordEmbeddingFeatureGroup {
	#[buffalo(id = 0, required)]
	pub source_column_name: String,
	#[buffalo(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[buffalo(id = 2, required)]
	pub model: WordEmbeddingModel,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct WordEmbeddingModel {
	/// This is the number of values for each word.
	#[buffalo(id = 0, required)]
	pub size: u64,
	/// This is a map from words to row indexes in `values`.
	#[buffalo(id = 1, required)]
	pub words: Vec<(String, u64)>,
	/// This holds the word embedding values, stored as a 2d array of shape (n_words, dimension) in row major order.
	#[buffalo(id = 2, required)]
	pub values: Vec<f32>,
}
