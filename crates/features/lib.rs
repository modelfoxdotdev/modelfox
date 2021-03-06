pub use self::{
	bag_of_words::BagOfWordsFeatureGroup,
	bag_of_words_cosine_similarity::BagOfWordsCosineSimilarityFeatureGroup,
	compute::{compute_features_array_f32, compute_features_array_value, compute_features_table},
	identity::IdentityFeatureGroup,
	normalized::NormalizedFeatureGroup,
	one_hot_encoded::OneHotEncodedFeatureGroup,
	word_embedding::WordEmbeddingFeatureGroup,
};

pub mod bag_of_words;
pub mod bag_of_words_cosine_similarity;
pub mod compute;
pub mod identity;
pub mod normalized;
pub mod one_hot_encoded;
pub mod word_embedding;

/// The `FeatureGroup` struct describes how to transform one or more columns from the input table to one or more columns in the output features.
#[derive(Clone, Debug)]
pub enum FeatureGroup {
	Identity(IdentityFeatureGroup),
	Normalized(NormalizedFeatureGroup),
	OneHotEncoded(OneHotEncodedFeatureGroup),
	BagOfWords(BagOfWordsFeatureGroup),
	WordEmbedding(WordEmbeddingFeatureGroup),
	BagOfWordsCosineSimilarity(BagOfWordsCosineSimilarityFeatureGroup),
}

impl FeatureGroup {
	/// Return the number of features this feature group will produce.
	pub fn n_features(&self) -> usize {
		match self {
			FeatureGroup::Identity(_) => 1,
			FeatureGroup::Normalized(_) => 1,
			FeatureGroup::OneHotEncoded(s) => s.variants.len() + 1,
			FeatureGroup::BagOfWords(s) => s.ngrams.len(),
			FeatureGroup::BagOfWordsCosineSimilarity(_) => 1,
			FeatureGroup::WordEmbedding(s) => s.model.size,
		}
	}
}
