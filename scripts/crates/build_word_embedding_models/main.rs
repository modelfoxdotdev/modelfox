use duct::cmd;
use ndarray::prelude::*;
use num::ToPrimitive;
use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};
use tangram_error::Result;

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
extern "C" {
	fn murmur2(key: *const u8, len: u64, seed: u64) -> u64;
}

pub fn main() -> Result<()> {
	let spacy_model_name = "en_core_web_md";
	let spacy_model_version = "3.0.0";
	let tar_path = PathBuf::from(format!(
		"/tmp/{0}-{1}.tar.gz",
		spacy_model_name, spacy_model_version
	));
	let untar_path = PathBuf::from(format!(
		"/tmp/{0}-{1}",
		spacy_model_name, spacy_model_version,
	));
	cmd!(
		"curl",
		"-sSL",
		format!(
			"https://github.com/explosion/spacy-models/releases/download/{0}-{1}/{0}-{1}.tar.gz",
			spacy_model_name, spacy_model_version
		)
	)
	.stdout_file(std::fs::File::create(&tar_path)?)
	.run()?;
	cmd!("tar", "xzvf", &tar_path, "--directory", "/tmp/",).run()?;
	let vocab_path = untar_path.join(format!(
		"{0}/{0}-{1}/vocab/",
		spacy_model_name, spacy_model_version,
	));
	let vocab_path = Path::new(&vocab_path);
	let key2row = std::fs::read(vocab_path.join("key2row"))?;
	let key2row: BTreeMap<u64, u64> = rmp_serde::from_slice(&key2row)?;
	let strings = std::fs::read(vocab_path.join("strings.json"))?;
	let strings: Vec<String> = serde_json::from_slice(&strings)?;
	let mut words = Vec::new();
	for string in strings {
		let hash = unsafe { murmur2(string.as_ptr(), string.len() as u64, 1) };
		if let Some(index) = key2row.get(&hash) {
			words.push((string, *index));
		}
	}
	let values: Array2<f32> = ndarray_npy::read_npy(vocab_path.join("vectors"))?;
	let mut writer = tangram_serialize::Writer::new();
	let size = values.ncols().to_u64().unwrap();
	let words = words
		.iter()
		.map(|(word, index)| (writer.write(word), *index))
		.collect::<Vec<_>>();
	let words = writer.write(&words);
	let values = writer.write(values.as_slice().unwrap());
	let model = writer.write(&tangram_model::WordEmbeddingModelWriter {
		size,
		words,
		values,
	});
	writer.write(&model);
	let word_embedding_models_dir = Path::new("data/word_embedding_models");
	std::fs::create_dir_all(&word_embedding_models_dir)?;
	std::fs::write(
		word_embedding_models_dir.join("en_core_web_md"),
		&writer.into_bytes(),
	)?;
	std::fs::remove_file(tar_path)?;
	std::fs::remove_dir_all(untar_path)?;
	Ok(())
}
