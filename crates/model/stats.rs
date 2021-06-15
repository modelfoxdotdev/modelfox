#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct StatsSettings {
	#[tangram_serialize(id = 0, required)]
	pub number_histogram_max_size: u64,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 8)]
pub enum ColumnStats {
	#[tangram_serialize(id = 0)]
	UnknownColumn(UnknownColumnStats),
	#[tangram_serialize(id = 1)]
	NumberColumn(NumberColumnStats),
	#[tangram_serialize(id = 2)]
	EnumColumn(EnumColumnStats),
	#[tangram_serialize(id = 3)]
	TextColumn(TextColumnStats),
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct UnknownColumnStats {
	#[tangram_serialize(id = 0, required)]
	pub column_name: String,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct NumberColumnStats {
	#[tangram_serialize(id = 0, required)]
	pub column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub invalid_count: u64,
	#[tangram_serialize(id = 2, required)]
	pub unique_count: u64,
	#[tangram_serialize(id = 3, required)]
	pub histogram: Option<Vec<(f32, u64)>>,
	#[tangram_serialize(id = 4, required)]
	pub min: f32,
	#[tangram_serialize(id = 5, required)]
	pub max: f32,
	#[tangram_serialize(id = 6, required)]
	pub mean: f32,
	#[tangram_serialize(id = 7, required)]
	pub variance: f32,
	#[tangram_serialize(id = 8, required)]
	pub std: f32,
	#[tangram_serialize(id = 9, required)]
	pub p25: f32,
	#[tangram_serialize(id = 10, required)]
	pub p50: f32,
	#[tangram_serialize(id = 11, required)]
	pub p75: f32,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct EnumColumnStats {
	#[tangram_serialize(id = 0, required)]
	pub column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub invalid_count: u64,
	#[tangram_serialize(id = 2, required)]
	pub histogram: Vec<(String, u64)>,
	#[tangram_serialize(id = 3, required)]
	pub unique_count: u64,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TextColumnStats {
	#[tangram_serialize(id = 0, required)]
	pub column_name: String,
	#[tangram_serialize(id = 1, required)]
	pub tokenizer: Tokenizer,
	#[tangram_serialize(id = 2, required)]
	pub ngram_types: Vec<NGramType>,
	#[tangram_serialize(id = 3, required)]
	pub ngrams_count: u64,
	#[tangram_serialize(id = 4, required)]
	pub top_ngrams: Vec<(NGram, TextColumnStatsTopNGramsEntry)>,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct Tokenizer {
	#[tangram_serialize(id = 0, required)]
	pub lowercase: bool,
	#[tangram_serialize(id = 1, required)]
	pub alphanumeric: bool,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub struct TextColumnStatsTopNGramsEntry {
	/// This is the number of examples with at least one occurrence of the ngram.
	#[tangram_serialize(id = 0, required)]
	pub row_count: u64,
	/// This is the number of occurrences of the ngram across all examples.
	#[tangram_serialize(id = 1, required)]
	pub occurrence_count: u64,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "static", value_size = 0)]
pub enum NGramType {
	#[tangram_serialize(id = 0)]
	Unigram,
	#[tangram_serialize(id = 1)]
	Bigram,
}

#[derive(tangram_serialize::Read, tangram_serialize::Write)]
#[tangram_serialize(size = "dynamic")]
pub enum NGram {
	#[tangram_serialize(id = 0)]
	Unigram(String),
	#[tangram_serialize(id = 1)]
	Bigram((String, String)),
}
