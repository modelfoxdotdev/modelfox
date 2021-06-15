#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct StatsSettings {
	#[buffalo(id = 0, required)]
	pub number_histogram_max_size: u64,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 8)]
pub enum ColumnStats {
	#[buffalo(id = 0)]
	UnknownColumn(UnknownColumnStats),
	#[buffalo(id = 1)]
	NumberColumn(NumberColumnStats),
	#[buffalo(id = 2)]
	EnumColumn(EnumColumnStats),
	#[buffalo(id = 3)]
	TextColumn(TextColumnStats),
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct UnknownColumnStats {
	#[buffalo(id = 0, required)]
	pub column_name: String,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct NumberColumnStats {
	#[buffalo(id = 0, required)]
	pub column_name: String,
	#[buffalo(id = 1, required)]
	pub invalid_count: u64,
	#[buffalo(id = 2, required)]
	pub unique_count: u64,
	#[buffalo(id = 3, required)]
	pub histogram: Option<Vec<(f32, u64)>>,
	#[buffalo(id = 4, required)]
	pub min: f32,
	#[buffalo(id = 5, required)]
	pub max: f32,
	#[buffalo(id = 6, required)]
	pub mean: f32,
	#[buffalo(id = 7, required)]
	pub variance: f32,
	#[buffalo(id = 8, required)]
	pub std: f32,
	#[buffalo(id = 9, required)]
	pub p25: f32,
	#[buffalo(id = 10, required)]
	pub p50: f32,
	#[buffalo(id = 11, required)]
	pub p75: f32,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct EnumColumnStats {
	#[buffalo(id = 0, required)]
	pub column_name: String,
	#[buffalo(id = 1, required)]
	pub invalid_count: u64,
	#[buffalo(id = 2, required)]
	pub histogram: Vec<(String, u64)>,
	#[buffalo(id = 3, required)]
	pub unique_count: u64,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TextColumnStats {
	#[buffalo(id = 0, required)]
	pub column_name: String,
	#[buffalo(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[buffalo(id = 2, required)]
	pub ngram_types: Vec<NGramType>,
	#[buffalo(id = 3, required)]
	pub ngrams_count: u64,
	#[buffalo(id = 4, required)]
	pub top_ngrams: Vec<(NGram, TextColumnStatsTopNGramsEntry)>,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Tokenizer {
	#[buffalo(id = 0, required)]
	pub lowercase: bool,
	#[buffalo(id = 1, required)]
	pub alphanumeric: bool,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct TextColumnStatsTopNGramsEntry {
	/// This is the number of examples with at least one occurrence of the ngram.
	#[buffalo(id = 0, required)]
	pub row_count: u64,
	/// This is the number of occurrences of the ngram across all examples.
	#[buffalo(id = 1, required)]
	pub occurrence_count: u64,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "static", value_size = 0)]
pub enum NGramType {
	#[buffalo(id = 0)]
	Unigram,
	#[buffalo(id = 1)]
	Bigram,
}

#[derive(buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub enum NGram {
	#[buffalo(id = 0)]
	Unigram(String),
	#[buffalo(id = 1)]
	Bigram((String, String)),
}
