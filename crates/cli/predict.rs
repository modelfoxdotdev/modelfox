use crate::PredictArgs;
use anyhow::Result;
use either::Either;
use itertools::Itertools;
use modelfox_core::predict::{PredictInput, PredictInputValue, PredictOptions};
use modelfox_zip::zip;

const PREDICT_CHUNK_SIZE: usize = 100;

pub fn predict(args: PredictArgs) -> Result<()> {
	let bytes = std::fs::read(&args.model)?;
	let model = modelfox_model::from_bytes(&bytes)?;
	let target_column_name = match model.inner() {
		modelfox_model::ModelInnerReader::Regressor(regressor) => {
			regressor.read().target_column_name()
		}
		modelfox_model::ModelInnerReader::BinaryClassifier(binary_classifier) => {
			binary_classifier.read().target_column_name()
		}
		modelfox_model::ModelInnerReader::MulticlassClassifier(multiclass_classifier) => {
			multiclass_classifier.read().target_column_name()
		}
	};
	let model = modelfox_core::predict::Model::from(model);
	let mut options = PredictOptions {
		compute_feature_contributions: false,
		..Default::default()
	};
	if let Some(threshold) = args.threshold {
		options.threshold = threshold;
	}
	let reader = match args.file {
		Some(path) => Either::Left(std::fs::File::open(path)?),
		None => Either::Right(std::io::stdin()),
	};
	let mut reader = csv::Reader::from_reader(reader);
	let writer = match args.output {
		Some(path) => Either::Left(std::fs::File::create(path)?),
		None => Either::Right(std::io::stdout()),
	};
	let mut writer = csv::Writer::from_writer(writer);
	let should_output_probabilies = args.probabilities.unwrap_or(false);
	match &model.inner {
		modelfox_core::predict::ModelInner::Regressor(_) => {
			writer.write_record(&[target_column_name])?;
		}
		modelfox_core::predict::ModelInner::BinaryClassifier(model) => {
			if should_output_probabilies {
				writer.write_record(&[
					model.positive_class.to_string(),
					model.negative_class.to_string(),
				])?;
			} else {
				writer.write_record(&[target_column_name])?;
			}
		}
		modelfox_core::predict::ModelInner::MulticlassClassifier(model) => {
			if should_output_probabilies {
				writer.write_record(
					&model
						.classes
						.iter()
						.map(|class| class.to_string())
						.collect::<Vec<_>>(),
				)?;
			} else {
				writer.write_record(&[target_column_name])?;
			}
		}
	};
	let header = reader.headers()?.to_owned();
	for records in &reader.records().chunks(PREDICT_CHUNK_SIZE) {
		let input: Vec<PredictInput> = records
			.into_iter()
			.map(|record| -> Result<PredictInput> {
				let record = record?;
				let input = zip!(header.iter(), record.into_iter())
					.map(|(column_name, value)| {
						(
							column_name.to_owned(),
							PredictInputValue::String(value.to_owned()),
						)
					})
					.collect();
				Ok(PredictInput(input))
			})
			.collect::<Result<_, _>>()?;
		let output = modelfox_core::predict::predict(&model, &input, &options);
		for output in output {
			let output = match output {
				modelfox_core::predict::PredictOutput::Regression(output) => {
					vec![output.value.to_string()]
				}
				modelfox_core::predict::PredictOutput::BinaryClassification(output) => {
					let model = match &model.inner {
						modelfox_core::predict::ModelInner::BinaryClassifier(model) => model,
						_ => {
							unreachable!()
						}
					};
					let class_name = output.class_name;
					let positive_class_probability = if class_name == model.positive_class {
						output.probability
					} else {
						1.0 - output.probability
					};
					let negative_class_probability = 1.0 - positive_class_probability;
					if should_output_probabilies {
						vec![
							positive_class_probability.to_string(),
							negative_class_probability.to_string(),
						]
					} else {
						vec![class_name]
					}
				}
				modelfox_core::predict::PredictOutput::MulticlassClassification(output) => {
					if should_output_probabilies {
						output
							.probabilities
							.iter()
							.map(|(_, probability)| probability.to_string())
							.collect()
					} else {
						vec![output.class_name]
					}
				}
			};
			writer.write_record(&output)?;
		}
	}
	Ok(())
}
