/*!
This crate implements the C API for libmodelfox, the ModelFox C library.
*/

#![allow(
	clippy::missing_safety_doc,
	clippy::upper_case_acronyms,
	non_camel_case_types
)]

use memmap::Mmap;
use std::{
	ffi::CStr,
	mem::transmute,
	os::raw::{c_char, c_double, c_float, c_void},
	panic::{catch_unwind, UnwindSafe},
	ptr::{null, null_mut},
};
type size_t = usize;

/// A `modelfox_string_view` value provides the pointer and length of a UTF-8 encoded string.
#[repr(C)]
pub struct modelfox_string_view {
	/// The pointer to the UTF-8 encoded bytes.
	ptr: *const c_char,
	/// The number bytes in the string.
	len: size_t,
}

impl modelfox_string_view {
	pub fn null() -> modelfox_string_view {
		modelfox_string_view {
			ptr: null(),
			len: 0,
		}
	}
}

impl From<&str> for modelfox_string_view {
	fn from(value: &str) -> Self {
		modelfox_string_view {
			ptr: value.as_ptr() as *const c_char,
			len: value.len(),
		}
	}
}

/// Retrieve the version of libmodelfox that is in use. On success, a string view of the version will be written to `version_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_version(version_ptr: *mut modelfox_string_view) {
	*version_ptr = env!("CARGO_PKG_VERSION").into();
}

/// A `modelfox_error` value is an opaque handle to an error returned by a libmodelfox function.
pub struct modelfox_error {
	message: String,
}

/// Delete an error.
#[no_mangle]
pub unsafe extern "C" fn modelfox_error_delete(error: *mut modelfox_error) {
	drop(Box::from_raw(error))
}

/// Retrieve an error message as a string view. The string view will be valid until `error` is deleted by calling `modelfox_error_delete`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_error_get_message(
	error: *mut modelfox_error,
	message_ptr: *mut modelfox_string_view,
) {
	*message_ptr = (*error).message.as_str().into();
}

// A `modelfox_model` value is an opaque handle to a model loaded by libmodelfox.
pub struct modelfox_model(modelfox_core::predict::Model);

/// Load a model from the file at `path`. On success, a pointer to the loaded model will be written to `model_ptr`. You must call `modelfox_model_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_from_path(
	path: *const c_char,
	model_ptr: *mut *const modelfox_model,
) -> *mut modelfox_error {
	handle_error(|| {
		let path = std::path::Path::new(CStr::from_ptr(path).to_str()?);
		let file = std::fs::File::open(path)?;
		let bytes = Mmap::map(&file)?;
		let model = ::modelfox_model::from_bytes(&bytes)?;
		let model = modelfox_core::predict::Model::from(model);
		*model_ptr = Box::into_raw(Box::new(modelfox_model(model)));
		Ok(())
	})
}

/// Load a model from the bytes pointed to by `model_data` with length `model_data_len`. On success, a pointer to the loaded model will be written to `model_ptr`. You must call `modelfox_model_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_from_bytes(
	model_bytes: *const c_void,
	model_bytes_len: size_t,
	model_ptr: *mut *const modelfox_model,
) -> *mut modelfox_error {
	handle_error(|| {
		let bytes = std::slice::from_raw_parts(model_bytes as *const u8, model_bytes_len);
		let model = ::modelfox_model::from_bytes(bytes)?;
		let model = modelfox_core::predict::Model::from(model);
		*model_ptr = Box::into_raw(Box::new(modelfox_model(model)));
		Ok(())
	})
}

/// Delete a model.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_delete(model: *mut modelfox_model) {
	drop(Box::from_raw(model));
}

/// Retrieve the id of a model. On success, a pointer to the model id as a modelfox string view will be written to `id_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_get_id(
	model: *const modelfox_model,
	id_ptr: *mut modelfox_string_view,
) {
	*id_ptr = (*model).0.id.as_str().into();
}

/// A `modelfox_task` identifies the task a model performs, one of regression, binary classification, or multiclass classification.
#[repr(C)]
pub enum modelfox_task {
	REGRESSION,
	BINARY_CLASSIFICATION,
	MULTICLASS_CLASSIFICATION,
}

/// Retrieve the task of the model. On success, the task will be written to `task_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_get_task(
	model: *const modelfox_model,
	task_ptr: *mut modelfox_task,
) {
	*task_ptr = match (*model).0.inner {
		modelfox_core::predict::ModelInner::Regressor(_) => modelfox_task::REGRESSION,
		modelfox_core::predict::ModelInner::BinaryClassifier(_) => {
			modelfox_task::BINARY_CLASSIFICATION
		}
		modelfox_core::predict::ModelInner::MulticlassClassifier(_) => {
			modelfox_task::MULTICLASS_CLASSIFICATION
		}
	};
}

/// A `modelfox_predict_input` is an opaque handle to a predict input to be passed to `modelfox_model_predict`.
pub struct modelfox_predict_input(modelfox_core::predict::PredictInput);

/// Create a new predict input. You must add it to a `modelfox_predict_input_vec` or call `modelfox_predict_input_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_new(
	predict_input_ptr: *mut *const modelfox_predict_input,
) {
	let predict_input = modelfox_core::predict::PredictInput::new();
	*predict_input_ptr = Box::into_raw(Box::new(modelfox_predict_input(predict_input)));
}

/// Delete a predict input.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_delete(predict_input: *mut modelfox_predict_input) {
	drop(Box::from_raw(predict_input))
}

/// Set the value of column `column_name` to the string `value`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_set_value_string(
	predict_input: *mut modelfox_predict_input,
	column_name: *const c_char,
	value: *const c_char,
) -> *mut modelfox_error {
	handle_error(|| {
		let column_name = CStr::from_ptr(column_name).to_str()?.to_owned();
		let value = CStr::from_ptr(value).to_str()?.to_owned();
		let value = modelfox_core::predict::PredictInputValue::String(value);
		((*predict_input).0).0.insert(column_name, value);
		Ok(())
	})
}

/// Set the value of column `column_name` to the number `value`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_set_value_number(
	predict_input: *mut modelfox_predict_input,
	column_name: *const c_char,
	value: c_double,
) -> *mut modelfox_error {
	handle_error(|| {
		let column_name = CStr::from_ptr(column_name).to_str()?.to_owned();
		let value = modelfox_core::predict::PredictInputValue::Number(value);
		((*predict_input).0).0.insert(column_name, value);
		Ok(())
	})
}

/// A `modelfox_predict_input_vec` is an opaque handle to a vec of predict inputs.
pub struct modelfox_predict_input_vec(Vec<modelfox_core::predict::PredictInput>);

/// Create a new predict input vec.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_vec_new(
	predict_input_vec: *mut *const modelfox_predict_input_vec,
) {
	*predict_input_vec = Box::into_raw(Box::new(modelfox_predict_input_vec(Vec::new())));
}

/// Delete a predict input vec.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_vec_delete(
	predict_input_vec: *mut modelfox_predict_input_vec,
) {
	drop(Box::from_raw(predict_input_vec));
}

/// Add a predict input to the predict input vec.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_input_vec_push(
	predict_input_vec: *mut modelfox_predict_input_vec,
	predict_input: *mut modelfox_predict_input,
) {
	(*predict_input_vec).0.push(Box::from_raw(predict_input).0);
}

/// A `modelfox_predict_options` value is an opaque handle to predict options to be passed to `modelfox_model_predict`.
pub struct modelfox_predict_options(modelfox_core::predict::PredictOptions);

/// Create a new `modelfox_predict_options` value. You must call `modelfox_predict_options_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_options_new(
	predict_options_ptr: *mut *const modelfox_predict_options,
) {
	*predict_options_ptr = Box::into_raw(Box::new(modelfox_predict_options(Default::default())));
}

/// Delete a `modelfox_predict_options` value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_options_delete(
	predict_options: *mut modelfox_predict_options,
) {
	drop(Box::from_raw(predict_options));
}

/// Set the classification threshold.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_options_set_threshold(
	predict_options: *mut modelfox_predict_options,
	threshold: c_float,
) {
	(*predict_options).0.threshold = threshold;
}

/// Enable or disable computing feature contributions.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_options_set_compute_feature_contributions(
	predict_options: *mut modelfox_predict_options,
	compute_feature_contributions: bool,
) {
	(*predict_options).0.compute_feature_contributions = compute_feature_contributions;
}

/// A `modelfox_predict_output_vec` is an opaque handle to a vec of predict outputs.
pub struct modelfox_predict_output_vec(Vec<modelfox_core::predict::PredictOutput>);

/// A `modelfox_predict_output` value is an opaque handle to the output of `modelfox_model_predict`.
pub struct modelfox_predict_output(modelfox_core::predict::PredictOutput);

/// A `modelfox_regression_predict_output` value is an opaque handle to a regression predict output returned by `modelfox_model_predict`.
pub struct modelfox_regression_predict_output(modelfox_core::predict::RegressionPredictOutput);

/// A `modelfox_binary_classification_predict_output` is an opaque handle to a binary classification predict output returned by `modelfox_model_predict`.
pub struct modelfox_binary_classification_predict_output(
	modelfox_core::predict::BinaryClassificationPredictOutput,
);

/// A `modelfox_multiclass_classification_predict_output` is an opaque handle to a multiclass classfication predict output returned by `modelfox_model_predict`.
pub struct modelfox_multiclass_classification_predict_output(
	modelfox_core::predict::MulticlassClassificationPredictOutput,
);

/// `modelfox_feature_contributions` is an opaque handle to the feature contributions returned from `modelfox_model_predict`.
pub struct modelfox_feature_contributions(modelfox_core::predict::FeatureContributions);

/// `modelfox_feature_contribution` is an opaque handle to a single modelfox feature contribution.
pub struct modelfox_feature_contribution_entry(modelfox_core::predict::FeatureContributionEntry);

/// `modelfox_identity_feature_contribution` is an opaque handle to a single modelfox identity feature contribution.
pub struct modelfox_identity_feature_contribution(
	modelfox_core::predict::IdentityFeatureContribution,
);

/// `modelfox_normalized_feature_contribution` is an opaque handle to a single modelfox normalized feature contribution.
pub struct modelfox_normalized_feature_contribution(
	modelfox_core::predict::NormalizedFeatureContribution,
);

/// `modelfox_one_hot_encoded_feature_contribution` is an opaque handle to a single modelfox one hot encoded feature contribution.
pub struct modelfox_one_hot_encoded_feature_contribution(
	modelfox_core::predict::OneHotEncodedFeatureContribution,
);

/// `modelfox_bag_of_words_feature_contribution` is an opaque handle to a single modelfox bag of words feature contribution.
pub struct modelfox_bag_of_words_feature_contribution(
	modelfox_core::predict::BagOfWordsFeatureContribution,
);

/// `modelfox_bag_of_words_cosine_similarity_feature_contribution` is an opaque handle to a single modelfox bag of words cosine similarity feature contribution.
pub struct modelfox_bag_of_words_cosine_similarity_feature_contribution(
	modelfox_core::predict::BagOfWordsCosineSimilarityFeatureContribution,
);

/// `modelfox_word_embedding_feature_contribution` is an opaque handle to a single modelfox word embedding feature contribution.
pub struct modelfox_word_embedding_feature_contribution(
	modelfox_core::predict::WordEmbeddingFeatureContribution,
);

/// `modelfox_ngram` is an opaque handle to an ngram.
pub struct modelfox_ngram(modelfox_core::predict::NGram);

/// `modelfox_ngram_type` identifies the ngram type.
#[repr(C)]
pub enum modelfox_ngram_type {
	UNIGRAM,
	BIGRAM,
}

/// `modelfox_unigram` is an opaque handle to unigram ngram.
pub struct modelfox_unigram(String);

/// `modelfox_bigram` is an opaque handle to bigram ngram.
pub struct modelfox_bigram((String, String));

/// Make a prediction! `model` should point to a model loaded with `modelfox_model_load`. `input` should be a `modelfox_predict_input` value and options should be a `modelfox_predict_options` value. On success, a pointer to a `modelfox_predict_output` output will be written to `output_ptr`. You must call `modelfox_predict_output_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_model_predict(
	model: *const modelfox_model,
	input: *const modelfox_predict_input_vec,
	options: *const modelfox_predict_options,
	output_ptr: *mut *const modelfox_predict_output_vec,
) -> *mut modelfox_error {
	handle_error(|| {
		let output = modelfox_core::predict::predict(
			&(*model).0,
			&input.as_ref().unwrap().0,
			&options.as_ref().unwrap().0,
		);
		*output_ptr = Box::into_raw(Box::new(modelfox_predict_output_vec(output)));
		Ok(())
	})
}

/// Delete a predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_delete(
	predict_output: *mut modelfox_predict_output,
) {
	drop(Box::from_raw(predict_output));
}

/// Delete a predict output vec.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_vec_delete(
	predict_output_vec: *mut modelfox_predict_output_vec,
) {
	drop(Box::from_raw(predict_output_vec));
}

/// Retrieve the len of the output vec.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_vec_len(
	predict_output_vec: *const modelfox_predict_output_vec,
	len_ptr: *mut size_t,
) {
	*len_ptr = (*predict_output_vec).0.len();
}

/// Get the predict output at `index`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_vec_get_at_index(
	predict_output_vec: *mut modelfox_predict_output_vec,
	index: size_t,
	predict_output_ptr: *mut *const modelfox_predict_output,
) {
	let predict_output = (*predict_output_vec).0.get(index).unwrap();
	*predict_output_ptr = transmute(predict_output);
}

/// Cast the predict output as `modelfox_regression_predict_output`. If this predict output is not for regression, null will be written to `regression_predict_output_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_as_regression(
	predict_output: *const modelfox_predict_output,
	predict_output_ptr: *mut *const modelfox_regression_predict_output,
) {
	*predict_output_ptr = match &(*predict_output).0 {
		modelfox_core::predict::PredictOutput::Regression(p) => transmute(p),
		modelfox_core::predict::PredictOutput::BinaryClassification(_) => null(),
		modelfox_core::predict::PredictOutput::MulticlassClassification(_) => null(),
	};
}

/// Cast the predict output as `modelfox_binary_classification_predict_output`. If this predict output is not for binary classification, null will be written to `binary_classification_predict_output_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_as_binary_classification(
	predict_output: *const modelfox_predict_output,
	predict_output_ptr: *mut *const modelfox_binary_classification_predict_output,
) {
	*predict_output_ptr = match &(*predict_output).0 {
		modelfox_core::predict::PredictOutput::Regression(_) => null(),
		modelfox_core::predict::PredictOutput::BinaryClassification(p) => transmute(p),
		modelfox_core::predict::PredictOutput::MulticlassClassification(_) => null(),
	};
}

/// Cast the predict output as `modelfox_multiclass_classification_predict_output`. If this predict output is not for multiclass classification, null will be written to `multiclass_classification_predict_output_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_predict_output_as_multiclass_classification(
	predict_output: *const modelfox_predict_output,
	predict_output_ptr: *mut *const modelfox_multiclass_classification_predict_output,
) {
	*predict_output_ptr = match &(*predict_output).0 {
		modelfox_core::predict::PredictOutput::Regression(_) => null(),
		modelfox_core::predict::PredictOutput::BinaryClassification(_) => null(),
		modelfox_core::predict::PredictOutput::MulticlassClassification(p) => transmute(p),
	};
}

/// Retrieve the value from a regression predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_regression_predict_output_get_value(
	predict_output: *const modelfox_regression_predict_output,
	output_value: *mut c_float,
) {
	*output_value = (*predict_output).0.value;
}

/// Retrieve the feature contributions from a regression predict output. If feature contributions were not computed for this prediction, null will be written to `feature_contributions_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_regression_predict_output_get_feature_contributions(
	predict_output: *const modelfox_regression_predict_output,
	feature_contributions_ptr: *mut *const modelfox_feature_contributions,
) {
	match &(*predict_output).0.feature_contributions {
		Some(feature_contributions) => {
			*feature_contributions_ptr = transmute(feature_contributions)
		}
		None => {
			*feature_contributions_ptr = null();
		}
	};
}

/// Retrieve the class name from a binary classification predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_binary_classification_predict_output_get_class_name(
	predict_output: *const modelfox_binary_classification_predict_output,
	class_name_ptr: *mut modelfox_string_view,
) {
	*class_name_ptr = (*predict_output).0.class_name.as_str().into();
}

/// Retrieve the probability from a binary classification predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_binary_classification_predict_output_get_probability(
	predict_output: *const modelfox_binary_classification_predict_output,
	probability: *mut c_float,
) {
	*probability = (*predict_output).0.probability;
}

/// Retrieve the feature contributions from a binary classification predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_binary_classification_predict_output_get_feature_contributions(
	predict_output: *const modelfox_binary_classification_predict_output,
	feature_contributions_ptr: *mut *const modelfox_feature_contributions,
) {
	match &(*predict_output).0.feature_contributions {
		Some(feature_contributions) => {
			*feature_contributions_ptr = transmute(feature_contributions)
		}
		None => {
			*feature_contributions_ptr = null();
		}
	};
}

/// Retrieve the class name from a multiclass classification predict output. If feature contributions were not computed for this prediction, null will be written to `feature_contributions_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_get_class_name(
	predict_output: *const modelfox_multiclass_classification_predict_output,
	class_name_ptr: *mut modelfox_string_view,
) {
	*class_name_ptr = (*predict_output).0.class_name.as_str().into();
}

/// Retrieve the probability from a multiclass classification predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_get_probability(
	predict_output: *const modelfox_multiclass_classification_predict_output,
	probability_ptr: *mut c_float,
) {
	*probability_ptr = (*predict_output).0.probability;
}

/// Retrieve the number of classes from a multiclass classification predict output.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_get_probabilities_len(
	predict_output: *const modelfox_multiclass_classification_predict_output,
	len_ptr: *mut size_t,
) {
	*len_ptr = (*predict_output).0.probabilities.len();
}

/// A `modelfox_multiclass_classification_predict_output_probabilities_iter` value is an iterator over `(class_name, probability)` pairs.
pub struct modelfox_multiclass_classification_predict_output_probabilities_iter<'a>(
	std::collections::btree_map::Iter<'a, String, f32>,
);

/// Delete a multiclass classification predict output probabilities iterator.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_probabilities_iter_delete(
	probabilities_iter: *mut modelfox_multiclass_classification_predict_output_probabilities_iter,
) {
	drop(Box::from_raw(probabilities_iter));
}

/// Get an iterator over the probabilities for a multiclass classification predict output. You must call `modelfox_multiclass_classification_predict_output_probabilities_iter_delete` when you are done with it.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_get_probabilities_iter(
	predict_output: *const modelfox_multiclass_classification_predict_output,
	probabilities_iter_ptr: *mut *const modelfox_multiclass_classification_predict_output_probabilities_iter,
) {
	*probabilities_iter_ptr = Box::into_raw(Box::new(
		modelfox_multiclass_classification_predict_output_probabilities_iter(
			(*predict_output).0.probabilities.iter(),
		),
	));
}

/// Retrieve the next `(class_name, probability)` pair from the probabilties iterator. This function returns `true` if `class_name_ptr` and `probability_ptr` have been successfully set, or `false` if the iterator has reached its end.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_probabilities_iter_next(
	probabilities_iter: *mut modelfox_multiclass_classification_predict_output_probabilities_iter,
	class_name_ptr: *mut modelfox_string_view,
	probability_ptr: *mut c_float,
) -> bool {
	match (*probabilities_iter).0.next() {
		Some((class_name, probability)) => {
			*class_name_ptr = class_name.as_str().into();
			*probability_ptr = *probability;
			true
		}
		None => false,
	}
}

/// A `modelfox_multiclass_classification_predict_output_feature_contributions_iter` value is an iterator over `(class_name, feature_contributions)` pairs.
pub struct modelfox_multiclass_classification_predict_output_feature_contributions_iter<'a>(
	std::collections::btree_map::Iter<'a, String, modelfox_core::predict::FeatureContributions>,
);

/// Delete a multiclass classification predict output feature contributions iterator.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_feature_contributions_iter_delete(
	feature_contributions_iter: *mut modelfox_multiclass_classification_predict_output_feature_contributions_iter,
) {
	drop(Box::from_raw(feature_contributions_iter));
}

/// Retrieve the feature contributions from a multiclass classification predict output. If feature contributions were not computed for this prediction, null will be written to `feature_contributions_iter_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_get_feature_contributions_iter(
	predict_output: *const modelfox_multiclass_classification_predict_output,
	feature_contributions_iter_ptr: *mut *const modelfox_multiclass_classification_predict_output_feature_contributions_iter,
) {
	match &(*predict_output).0.feature_contributions {
		Some(feature_contributions) => {
			*feature_contributions_iter_ptr = Box::into_raw(Box::new(
				modelfox_multiclass_classification_predict_output_feature_contributions_iter(
					feature_contributions.iter(),
				),
			));
		}
		None => {
			*feature_contributions_iter_ptr = null();
		}
	};
}

/// Retrieve the next `(class_name, feature_contributions)` pair from the feature contributions iterator. This function returns `true` if `class_name_ptr` and `feature_contributions_ptr` have been successfully set, or `false` if the iterator has reached its end.
#[no_mangle]
pub unsafe extern "C" fn modelfox_multiclass_classification_predict_output_feature_contributions_iter_next(
	feature_contributions_iter: *mut modelfox_multiclass_classification_predict_output_feature_contributions_iter,
	class_name_ptr: *mut modelfox_string_view,
	feature_contributions_ptr: *mut *const modelfox_feature_contributions,
) -> bool {
	match (*feature_contributions_iter).0.next() {
		Some((class_name, feature_contributions)) => {
			*class_name_ptr = class_name.as_str().into();
			*feature_contributions_ptr = transmute(feature_contributions);
			true
		}
		None => false,
	}
}

/// Retrieve the baseline value from feature contributions.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contributions_get_baseline_value(
	feature_contributions: *const modelfox_feature_contributions,
	baseline_value_ptr: *mut c_float,
) {
	*baseline_value_ptr = (*feature_contributions).0.baseline_value;
}

/// Retrieve the output value from feature contributions.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contributions_get_output_value(
	feature_contributions: *const modelfox_feature_contributions,
	output_value_ptr: *mut c_float,
) {
	*output_value_ptr = (*feature_contributions).0.output_value;
}

/// Retrieve the len of the feature contributions.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contributions_get_entries_len(
	feature_contributions: *const modelfox_feature_contributions,
	len_ptr: *mut size_t,
) {
	*len_ptr = (*feature_contributions).0.entries.len();
}

/// Retrieve the feature contribution at `index`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contributions_get_entry_at_index(
	feature_contributions: *const modelfox_feature_contributions,
	index: size_t,
	feature_contribution_entry_ptr: *mut *const modelfox_feature_contribution_entry,
) {
	*feature_contribution_entry_ptr = (*feature_contributions)
		.0
		.entries
		.get(index)
		.map(|value| transmute(value))
		.unwrap_or_else(null);
}

/// `modelfox_feature_contribution_type` corresponds to the ModelFox feature type.
#[repr(C)]
pub enum modelfox_feature_contribution_entry_type {
	IDENTITY,
	NORMALIZED,
	ONE_HOT_ENCODED,
	BAG_OF_WORDS,
	BAG_OF_WORDS_COSINE_SIMILARITY,
	WORD_EMBEDDING,
}

/// Retrieve the type of the feature contribution entry.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_get_type(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	type_ptr: *mut modelfox_feature_contribution_entry_type,
) {
	*type_ptr = match (*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => {
			modelfox_feature_contribution_entry_type::IDENTITY
		}
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => {
			modelfox_feature_contribution_entry_type::NORMALIZED
		}
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => {
			modelfox_feature_contribution_entry_type::ONE_HOT_ENCODED
		}
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => {
			modelfox_feature_contribution_entry_type::BAG_OF_WORDS
		}
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => {
			modelfox_feature_contribution_entry_type::BAG_OF_WORDS_COSINE_SIMILARITY
		}
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => {
			modelfox_feature_contribution_entry_type::WORD_EMBEDDING
		}
	}
}

/// Cast the feature contribution entry as `modelfox_identity_feature_contribution`. If this feature contribution is not an identity feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_identity(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_identity_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(f) => transmute(f),
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => null(),
	};
}

/// Cast the feature contribution entry as `modelfox_normalized_feature_contribution`. If this feature contribution is not a normalized feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_normalized(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_normalized_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::Normalized(f) => transmute(f),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => null(),
	};
}

/// Cast the feature contribution entry as `modelfox_one_hot_encoded_feature_contribution`. If this feature contribution is not a one hot encoded feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_one_hot_encoded(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_one_hot_encoded_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(f) => transmute(f),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => null(),
	};
}

/// Cast the feature contribution entry as `modelfox_bag_of_words_feature_contribution`. If this feature contribution is not a bag of words feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_bag_of_words(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_bag_of_words_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(f) => transmute(f),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => null(),
	};
}

/// Cast the feature contribution entry as `modelfox_bag_of_words_cosine_similarity_feature_contribution`. If this feature contribution is not a bag of words cosine similarity feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_bag_of_words_cosine_similarity(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_bag_of_words_cosine_similarity_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(f) => {
			transmute(f)
		}
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(_) => null(),
	};
}

/// Cast the feature contribution entry as `modelfox_word_embedding_feature_contribution`. If this feature contribution is not a word embedding feature contribution, null will be written to `feature_contribution_ouput_ptr`.
#[no_mangle]
pub unsafe extern "C" fn modelfox_feature_contribution_entry_as_word_embedding(
	feature_contribution_entry: *const modelfox_feature_contribution_entry,
	feature_contribution_ptr: *mut *const modelfox_word_embedding_feature_contribution,
) {
	*feature_contribution_ptr = match &(*feature_contribution_entry).0 {
		modelfox_core::predict::FeatureContributionEntry::Identity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::Normalized(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::OneHotEncoded(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWords(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::BagOfWordsCosineSimilarity(_) => null(),
		modelfox_core::predict::FeatureContributionEntry::WordEmbedding(f) => transmute(f),
	};
}

/// Retrieve the column name.
#[no_mangle]
pub unsafe extern "C" fn modelfox_identity_feature_contribution_get_column_name(
	feature_contribution: *const modelfox_identity_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name.as_str().into();
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_identity_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_identity_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// Retrieve the feature value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_identity_feature_contribution_get_feature_value(
	feature_contribution: *const modelfox_identity_feature_contribution,
	feature_value: *mut c_float,
) {
	*feature_value = (*feature_contribution).0.feature_value;
}

/// Retrieve the column name.
#[no_mangle]
pub unsafe extern "C" fn modelfox_normalized_feature_contribution_get_column_name(
	feature_contribution: *const modelfox_normalized_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name.as_str().into();
}

/// Retrieve the feature value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_normalized_feature_contribution_get_feature_value(
	feature_contribution: *const modelfox_normalized_feature_contribution,
	feature_value: *mut c_float,
) {
	*feature_value = (*feature_contribution).0.feature_value;
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_normalized_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_normalized_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// Retrieve the column name.
#[no_mangle]
pub unsafe extern "C" fn modelfox_one_hot_encoded_feature_contribution_get_column_name(
	feature_contribution: *const modelfox_one_hot_encoded_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name.as_str().into();
}

/// Retrieve the variant.
#[no_mangle]
pub unsafe extern "C" fn modelfox_one_hot_encoded_feature_contribution_get_variant(
	feature_contribution: *const modelfox_one_hot_encoded_feature_contribution,
	variant_ptr: *mut modelfox_string_view,
) {
	*variant_ptr = match &(*feature_contribution).0.variant {
		Some(variant) => variant.as_str().into(),
		None => modelfox_string_view::null(),
	};
}

/// Retrieve the feature value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_one_hot_encoded_feature_contribution_get_feature_value(
	feature_contribution: *const modelfox_one_hot_encoded_feature_contribution,
	feature_value: *mut bool,
) {
	*feature_value = (*feature_contribution).0.feature_value;
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_one_hot_encoded_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_one_hot_encoded_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// Retrieve the column name.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_feature_contribution_get_column_name(
	feature_contribution: *const modelfox_bag_of_words_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name.as_str().into();
}

/// Retrieve the ngram.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_feature_contribution_get_ngram(
	feature_contribution: *const modelfox_bag_of_words_feature_contribution,
	ngram_ptr: *mut *const modelfox_ngram,
) {
	*ngram_ptr = transmute(&(*feature_contribution).0.ngram);
}

/// Retrieve the ngram type.
#[no_mangle]
pub unsafe extern "C" fn modelfox_ngram_get_type(
	ngram: *const modelfox_ngram,
	ngram_type: *mut modelfox_ngram_type,
) {
	*ngram_type = match (*ngram).0 {
		modelfox_core::predict::NGram::Unigram(_) => modelfox_ngram_type::UNIGRAM,
		modelfox_core::predict::NGram::Bigram(_, _) => modelfox_ngram_type::BIGRAM,
	}
}

/// Retrieve the unigram token.
#[no_mangle]
pub unsafe extern "C" fn modelfox_unigram_get_token(
	ngram: *const modelfox_ngram,
	token_ptr: *mut modelfox_string_view,
) {
	*token_ptr = match &(*ngram).0 {
		modelfox_core::predict::NGram::Unigram(token) => token.as_str().into(),
		modelfox_core::predict::NGram::Bigram(_, _) => modelfox_string_view::null(),
	};
}

/// Retrieve the bigram token a.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bigram_get_token_a(
	ngram: *const modelfox_ngram,
	token_ptr: *mut modelfox_string_view,
) {
	*token_ptr = match &(*ngram).0 {
		modelfox_core::predict::NGram::Unigram(_) => modelfox_string_view::null(),
		modelfox_core::predict::NGram::Bigram(token_a, _) => token_a.as_str().into(),
	};
}

/// Retrieve the bigram token b.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bigram_get_token_b(
	ngram: *const modelfox_ngram,
	token_ptr: *mut modelfox_string_view,
) {
	*token_ptr = match &(*ngram).0 {
		modelfox_core::predict::NGram::Unigram(_) => modelfox_string_view::null(),
		modelfox_core::predict::NGram::Bigram(_, token_b) => token_b.as_str().into(),
	};
}

/// Retrieve the feature value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_feature_contribution_get_feature_value(
	feature_contribution: *const modelfox_bag_of_words_feature_contribution,
	feature_value: *mut c_float,
) {
	*feature_value = (*feature_contribution).0.feature_value;
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_bag_of_words_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// Retrieve the column name a.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_cosine_similarity_feature_contribution_get_column_name_a(
	feature_contribution: *const modelfox_bag_of_words_cosine_similarity_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name_a.as_str().into();
}

/// Retrieve the column name b.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_cosine_similarity_feature_contribution_get_column_name_b(
	feature_contribution: *const modelfox_bag_of_words_cosine_similarity_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name_b.as_str().into();
}

/// Retrieve the feature value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_cosine_similarity_feature_contribution_get_feature_value(
	feature_contribution: *const modelfox_bag_of_words_cosine_similarity_feature_contribution,
	feature_value: *mut c_float,
) {
	*feature_value = (*feature_contribution).0.feature_value;
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_bag_of_words_cosine_similarity_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_bag_of_words_cosine_similarity_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// Retrieve the column name.
#[no_mangle]
pub unsafe extern "C" fn modelfox_word_embedding_feature_contribution_get_column_name(
	feature_contribution: *const modelfox_word_embedding_feature_contribution,
	column_name_ptr: *mut modelfox_string_view,
) {
	*column_name_ptr = (*feature_contribution).0.column_name.as_str().into();
}

/// Retrieve the value index.
#[no_mangle]
pub unsafe extern "C" fn modelfox_word_embedding_feature_contribution_get_value_index(
	feature_contribution: *const modelfox_word_embedding_feature_contribution,
	value_index: *mut size_t,
) {
	*value_index = (*feature_contribution).0.value_index;
}

/// Retrieve the feature contribution value.
#[no_mangle]
pub unsafe extern "C" fn modelfox_word_embedding_feature_contribution_get_feature_contribution_value(
	feature_contribution: *const modelfox_word_embedding_feature_contribution,
	feature_contribution_value: *mut c_float,
) {
	*feature_contribution_value = (*feature_contribution).0.feature_contribution_value;
}

/// This function exposes the allocator used by libmodelfox. It is used by the wasm build of libmodelfox because WebAssembly does not include its own allocator.
#[no_mangle]
pub unsafe extern "C" fn modelfox_alloc(size: size_t, align: size_t) -> *mut c_void {
	let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
	std::alloc::alloc(layout) as *mut c_void
}

/// This function exposes the allocator used by libmodelfox. It is used by the wasm build of libmodelfox because WebAssembly does not include its own allocator.
#[no_mangle]
pub unsafe extern "C" fn modelfox_dealloc(ptr: *mut c_void, size: size_t, align: size_t) {
	if size == 0 {
		return;
	}
	let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
	std::alloc::dealloc(ptr as *mut u8, layout);
}

fn handle_error<F>(f: F) -> *mut modelfox_error
where
	F: FnOnce() -> ::anyhow::Result<()> + UnwindSafe,
{
	match catch_unwind(f) {
		Ok(Ok(_)) => null_mut(),
		Ok(Err(error)) => Box::into_raw(Box::new(modelfox_error {
			message: error.to_string(),
		})),
		Err(_) => Box::into_raw(Box::new(modelfox_error {
			message: "A panic occurred".to_owned(),
		})),
	}
}
