package tangram

// #cgo linux,amd64 CFLAGS: -I${SRCDIR}/libtangram/x86_64-unknown-linux-musl
// #cgo linux,amd64 LDFLAGS: -L${SRCDIR}/libtangram/x86_64-unknown-linux-musl -ltangram -ldl -lm
// #cgo linux,arm64 CFLAGS: -I${SRCDIR}/libtangram/aarch64-unknown-linux-musl
// #cgo linux,arm64 LDFLAGS: -L${SRCDIR}/libtangram/aarch64-unknown-linux-musl -ltangram -ldl -lm
// #cgo darwin,amd64 CFLAGS: -I${SRCDIR}/libtangram/x86_64-apple-darwin
// #cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/libtangram/x86_64-apple-darwin -ltangram
// #cgo darwin,arm64 CFLAGS: -I${SRCDIR}/libtangram/aarch64-apple-darwin
// #cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/libtangram/aarch64-apple-darwin -ltangram
// #cgo windows,amd64 CFLAGS: -I${SRCDIR}/libtangram/x86_64-pc-windows-gnu
// #cgo windows,amd64 LDFLAGS: -L${SRCDIR}/libtangram/x86_64-pc-windows-gnu -ltangram -luserenv -lws2_32
// #include "tangram.h"
// #include <stdlib.h>
import "C"

import (
	"bytes"
	"encoding/json"
	"errors"
	"io/ioutil"
	"log"
	"net/http"
	"strconv"
	"time"
	"unsafe"
)

// Use this struct to load a model, make predictions, and log events to the app.
type Model struct {
	modelPtr *C.tangram_model
	options  *LoadModelOptions
	logQueue []event
}

// These are the options passed when loading a model.
type LoadModelOptions struct {
	// If you are running the app locally or on your own server, use this field to provide the url to it. If not specified, the default value is https://app.tangram.xyz.
	TangramURL string
}

// These are the options passed to `Predict`.
type PredictOptions struct {
	// If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.
	Threshold float32 `json:"threshold"`
	// Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `feature_contributions` field of the predict output.
	ComputeFeatureContributions bool `json:"computeFeatureContributions"`
}

// This is the input type of `Predict`. A predict input is a map from strings to strings or floats. The keys should match the columns in the CSV file you trained your model with.
type PredictInput map[string]interface{}

// TaskType is the type of the task corresponding to the model task, one of RegressionTaskType, BinaryClassificationTaskType, and MulticlassClassificationTaskType.
type TaskType int

const (
	RegressionTaskType = iota
	BinaryClassificationTaskType
	MulticlassClassificationTaskType
)

// This is the return type of `Predict`.
type PredictOutput interface {
	isPredictOutput()
}

func (RegressionPredictOutput) isPredictOutput()               {}
func (BinaryClassificationPredictOutput) isPredictOutput()     {}
func (MulticlassClassificationPredictOutput) isPredictOutput() {}

// `Predict` outputs `RegressionPredictOutput` when the model's task is regression.
type RegressionPredictOutput struct {
	// This is the predicted value.
	Value float32 `json:"value"`
	// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	FeatureContributions FeatureContributions `json:"-"`
}

// `Predict` outputs `BinaryClassificationPredictOutput` when the model's task is regression.
type BinaryClassificationPredictOutput struct {
	// This is the name of the predicted class.
	ClassName string `json:"className"`
	// This is the probability the model assigned to the predicted class.
	Probability float32 `json:"probability"`
	// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	FeatureContributions FeatureContributions `json:"-"`
}

// `Predict` outputs `MulticlassClassificationPredictOutput` when the model's task is regression.
type MulticlassClassificationPredictOutput struct {
	// This is the name of the predicted class.
	ClassName string `json:"className`
	// This is the probability the model assigned to the predicted class.
	Probability float32 `json:"probability"`
	// This value maps from class names to the probability the model assigned to each class.
	Probabilities map[string]float32 `json:"probabilities"`
	// If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `output_value` is the predicted class.
	FeatureContributions map[string]FeatureContributions `json:"-"`
}

// This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.
type FeatureContributions struct {
	// This is the value the model would output if all features had baseline values.
	BaselineValue float32
	// This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.
	OutputValue float32
	// This list will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
	Entries []FeatureContributionEntry
}

// This identifies the type of a feature contribution.
type FeatureContributionType int

const (
	// IdentityFeatureContributionType is the feature contribution type of an identity feature group.
	IdentityFeatureContributionType = iota
	// NormalizedFeatureContributionType is the feature contribution type of a normalized feature group.
	NormalizedFeatureContributionType
	// OneHotEncodedFeatureContributionType is the feature contribution type of a one hot encoded feature group.
	OneHotEncodedFeatureContributionType
	// BagOfWordsFeatureContributionType is the feature contribution type of a bag of words feature group.
	BagOfWordsFeatureContributionType
	// BagOfWordsCosineSimilarityFeatureContributionType is the feature contribution type of a bag of words cosine similarity feature group.
	BagOfWordsCosineSimilarityFeatureContributionType
	// WordEmbeddingFeatureContributionType is the feature contribution type of a word embedding feature group.
	WordEmbeddingFeatureContributionType
)

// FeatureContribution represents a feature contribution.
type FeatureContributionEntry interface {
	isFeatureContribution()
}

func (IdentityFeatureContribution) isFeatureContribution()      {}
func (NormalizedFeatureContribution) isFeatureContribution()    {}
func (OneHotEncodedFeatureContribution) isFeatureContribution() {}
func (BagOfWordsFeatureContribution) isFeatureContribution()    {}
func (BagOfWordsCosineSimilarityFeatureContribution) isFeatureContribution()    {}
func (WordEmbeddingFeatureContribution) isFeatureContribution() {}

// This describes the contribution of a feature from an identity feature group
type IdentityFeatureContribution struct {
	// This is the name of the source column for the feature group.
	ColumnName string
	// This is the value of the feature.
	FeatureValue float32
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// This describes the contribution of a feature from a normalized feature group.
type NormalizedFeatureContribution struct {
	// This is the name of the source column for the feature group.
	ColumnName string
	// This is the value of the feature.
	FeatureValue float32
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// This describes the contribution of a feature from a one hot encoded feature group.
type OneHotEncodedFeatureContribution struct {
	// This is the name of the source column for the feature group.
	ColumnName string
	// This is the enum variant the feature indicates the presence of.
	Variant string
	// This is the value of the feature.
	FeatureValue bool
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// This describes the contribution of a feature from a bag of words feature group.
type BagOfWordsFeatureContribution struct {
	// This is the name of the source column for the feature group.
	ColumnName string
	// This is the ngram for the feature.
	NGram NGram
	// This is the value of the feature.
	FeatureValue float32
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// NGram is the token type in a bag of words feature group.
type NGram interface {
	isNgram()
}

func (Unigram) isNgram() {}
func (Bigram) isNgram()  {}

// This describes a unigram ngram.
type Unigram struct {
	// This is the token.
	Token string
}

// This describes a bigram ngram.
type Bigram struct {
	// This is the first token in the bigram.
	TokenA string
	// This is the second token in the bigram.
	TokenB string
}

// This describes the contribution of a feature from a bag of words cosine similarity feature group.
type BagOfWordsCosineSimilarityFeatureContribution struct {
	// This is the name of the source column a for the feature group.
	ColumnNameA string
	// This is the name of the source column b for the feature group.
	ColumnNameB string
	// This is the value of the feature.
	FeatureValue float32
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// This describes the contribution of a feature from a word embedding feature group.
type WordEmbeddingFeatureContribution struct {
	// This is the name of the source column for the feature group.
	ColumnName string
	// This is the index of the feature in the word embedding.
	ValueIndex int
	// This is the amount that the feature contributed to the output.
	FeatureContributionValue float32
}

// This is the type of the argument to `model.LogPrediction` and `model.EnqueueLogPrediction` which specifies the details of the prediction to log.
type LogPredictionArgs struct {
	// This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
	Identifier string
	// This is the same `PredictInput` value that you passed to `model.Predict`.
	Input PredictInput
	// This is the same `PredictOptions` value that you passed to `model.Predict`.
	Options PredictOptions
	// This is the output returned by `model.Predict`.
	Output PredictOutput
}

// This is the type of the argument to `model.LogTrueValue` and `model.EnqueueLogTrueValue` which specifies the details of the true value to log.
type LogTrueValueArgs struct {
	// This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
	Identifier string
	// This is the true value for the prediction.
	TrueValue interface{}
}

type event map[string]interface{}

// This is the version of libtangram that is in use.
func Version() string {
	var s C.tangram_string_view
	C.tangram_version(&s)
	id := C.GoStringN(s.ptr, C.int(s.len))
	return id
}

// Load a model from a `.tangram` file at `path`.
func LoadModelFromPath(path string, options *LoadModelOptions) (*Model, error) {
	var cModel *C.tangram_model
	cPath := C.CString(path)
	err := C.tangram_model_from_path(cPath, &cModel)
	if err != nil {
		var sv C.tangram_string_view
		defer C.tangram_error_delete(err)
		C.tangram_error_get_message(err, &sv)
		errs := C.GoStringN(sv.ptr, C.int(sv.len))
		return nil, errors.New(errs)
	}
	queue := []event{}
	model := Model{
		cModel,
		options,
		queue,
	}
	return &model, nil
}

// Load a model from bytes instead of a file. You should use this only if you already have a `.tangram` loaded into memory. Otherwise, use `model.LoadModelFromPath`, which is faster because it memory maps the file.
func LoadModelFromBytes(data []byte, options *LoadModelOptions) (*Model, error) {
	var cModel *C.tangram_model
	cBytes := C.CBytes(data)
	cLen := C.size_t(len(data))
	err := C.tangram_model_from_bytes(cBytes, cLen, &cModel)
	if err != nil {
		var sv C.tangram_string_view
		defer C.tangram_error_delete(err)
		C.tangram_error_get_message(err, &sv)
		errs := C.GoStringN(sv.ptr, C.int(sv.len))
		return nil, errors.New(errs)
	}
	queue := []event{}
	model := Model{
		cModel,
		options,
		queue,
	}
	return &model, nil
}

// Destroy frees up the memory used by the model. You should call this with defer after loading your model.
func (m Model) Destroy() {
	C.tangram_model_delete(m.modelPtr)
}

// Retrieve the model's id.
func (m Model) ID() string {
	var sv C.tangram_string_view
	C.tangram_model_get_id(m.modelPtr, &sv)
	id := C.GoStringN(sv.ptr, C.int(sv.len))
	return id
}

func newPredictInputVec(inputVec []PredictInput) *C.tangram_predict_input_vec {
	var cInputVec *C.tangram_predict_input_vec
	C.tangram_predict_input_vec_new(&cInputVec)
	for i := 0; i < len(inputVec); i++ {
		cInput := newPredictInput(inputVec[i])
		C.tangram_predict_input_vec_push(cInputVec, cInput)
	}
	return cInputVec
}

func newPredictInput(input PredictInput) *C.tangram_predict_input {
	var cInput *C.tangram_predict_input
	C.tangram_predict_input_new(&cInput)
	var cKey *C.char
	defer C.free(unsafe.Pointer(cKey))
	for key, value := range input {
		switch value.(type) {
		case string:
			cKey = C.CString(key)
			cVal := C.CString(value.(string))
			err := C.tangram_predict_input_set_value_string(cInput, cKey, cVal)
			if err != nil {
				logTangramError(err)
			}
		case float64:
			cKey = C.CString(key)
			cVal := C.double(float64(value.(float64)))
			err := C.tangram_predict_input_set_value_number(cInput, cKey, cVal)
			if err != nil {
				logTangramError(err)
			}
		case int:
			cKey = C.CString(key)
			cVal := C.double(float64(value.(int)))
			err := C.tangram_predict_input_set_value_number(cInput, cKey, cVal)
			if err != nil {
				logTangramError(err)
			}
		case bool:
			cKey = C.CString(key)
			cVal := C.CString(strconv.FormatBool(value.(bool)))
			err := C.tangram_predict_input_set_value_string(cInput, cKey, cVal)
			if err != nil {
				logTangramError(err)
			}
		}
	}
	return cInput
}

func newPredictOptions(predictOptions *PredictOptions) *C.tangram_predict_options {
	var cPredictOptions *C.tangram_predict_options
	C.tangram_predict_options_new(&cPredictOptions)
	if predictOptions != nil {
		C.tangram_predict_options_set_threshold(cPredictOptions, C.float(predictOptions.Threshold))

		C.tangram_predict_options_set_compute_feature_contributions(cPredictOptions, C.bool(predictOptions.ComputeFeatureContributions))
	}
	return cPredictOptions
}

// Make a prediction with a single input.
func (m Model) PredictOne(input PredictInput, options *PredictOptions) PredictOutput {
	return m.Predict([]PredictInput{input}, options)[0]
}

func logTangramError(cErr *C.tangram_error) {
	var sv C.tangram_string_view
	defer C.tangram_error_delete(cErr)
	C.tangram_error_get_message(cErr, &sv)
	err := C.GoStringN(sv.ptr, C.int(sv.len))
	log.Fatal(err)
}

// Make a prediction with multiple inputs.
func (m Model) Predict(input []PredictInput, options *PredictOptions) []PredictOutput {
	var cOutputVec *C.tangram_predict_output_vec
	cInputVec := newPredictInputVec(input)
	cOptions := newPredictOptions(options)
	defer C.tangram_predict_options_delete(cOptions)
	defer C.tangram_predict_input_vec_delete(cInputVec)
	err := C.tangram_model_predict(m.modelPtr, cInputVec, cOptions, &cOutputVec)
	if err != nil {
		logTangramError(err)
	}
	defer C.tangram_predict_output_vec_delete(cOutputVec)

	outputVec := make([]PredictOutput, len(input))
	var cTaskType C.tangram_task
	C.tangram_model_get_task(m.modelPtr, &cTaskType)
	for i := 0; i < len(input); i++ {
		var cOutput *C.tangram_predict_output
		C.tangram_predict_output_vec_get_at_index(cOutputVec, C.size_t(i), &cOutput)
		outputVec[i] = makePredictOutputFromTangramPredictOutput(cTaskType, cOutput)
	}
	return outputVec
}

// A helper function to extract a PredictOutput from a *C.tangram_predict_output.
func makePredictOutputFromTangramPredictOutput(taskType C.tangram_task, cOutput *C.tangram_predict_output) PredictOutput {
	switch taskType {
	case RegressionTaskType:
		return makeRegressionPredictOutputFromTangramPredictOutput(cOutput)
	case BinaryClassificationTaskType:
		return makeBinaryClassificationPredictOutputFromTangramPredictOutput(cOutput)
	case MulticlassClassificationTaskType:
		return makeMulticlassClassificationPredictOutputFromTangramPredictOutput(cOutput)
	default:
		log.Fatal("tangram error")
	}
	return nil
}

// A helper function to extract a RegressionPredictOutput from a *C.tangram_predict_output.
func makeRegressionPredictOutputFromTangramPredictOutput(output *C.tangram_predict_output) RegressionPredictOutput {
	var cOutput *C.tangram_regression_predict_output
	var cValue C.float
	C.tangram_predict_output_as_regression(output, &cOutput)
	C.tangram_regression_predict_output_get_value(cOutput, &cValue)
	var fcs FeatureContributions
	var cFeatureContributions *C.tangram_feature_contributions
	C.tangram_regression_predict_output_get_feature_contributions(cOutput, &cFeatureContributions)
	if cFeatureContributions != nil {
		fcs = makeFeatureContributions(cFeatureContributions)
	}
	return RegressionPredictOutput{
		Value:                float32(cValue),
		FeatureContributions: fcs,
	}
}

// A helper function to extract a BinaryClassificationPredictOutput from a *C.tangram_predict_output.
func makeBinaryClassificationPredictOutputFromTangramPredictOutput(output *C.tangram_predict_output) BinaryClassificationPredictOutput {
	var cOutput *C.tangram_binary_classification_predict_output
	var cProbability C.float
	var sv C.tangram_string_view
	C.tangram_predict_output_as_binary_classification(output, &cOutput)
	C.tangram_binary_classification_predict_output_get_probability(cOutput, &cProbability)
	C.tangram_binary_classification_predict_output_get_class_name(cOutput, &sv)
	className := C.GoStringN(sv.ptr, C.int(sv.len))
	var fcs FeatureContributions
	var cFeatureContributions *C.tangram_feature_contributions
	C.tangram_binary_classification_predict_output_get_feature_contributions(cOutput, &cFeatureContributions)
	if cFeatureContributions != nil {
		fcs = makeFeatureContributions(cFeatureContributions)
	}
	return BinaryClassificationPredictOutput{
		ClassName:            className,
		Probability:          float32(cProbability),
		FeatureContributions: fcs,
	}
}

// A helper function to extract a MulticlassClassificationPredictOutput from a *C.tangram_predict_output.
func makeMulticlassClassificationPredictOutputFromTangramPredictOutput(output *C.tangram_predict_output) MulticlassClassificationPredictOutput {
	var cOutput *C.tangram_multiclass_classification_predict_output
	var cProbability C.float
	var sv C.tangram_string_view
	C.tangram_predict_output_as_multiclass_classification(output, &cOutput)
	C.tangram_multiclass_classification_predict_output_get_probability(cOutput, &cProbability)
	C.tangram_multiclass_classification_predict_output_get_class_name(cOutput, &sv)
	predictedClassName := C.GoStringN(sv.ptr, C.int(sv.len))
	var cClassProbability C.float
	var cProbabilitiesIter *C.tangram_multiclass_classification_predict_output_probabilities_iter
	C.tangram_multiclass_classification_predict_output_get_probabilities_iter(cOutput, &cProbabilitiesIter)
	defer C.tangram_multiclass_classification_predict_output_probabilities_iter_delete(cProbabilitiesIter)
	probabilities := make(map[string]float32)
	for C.tangram_multiclass_classification_predict_output_probabilities_iter_next(cProbabilitiesIter, &sv, &cClassProbability) {
		className := C.GoStringN(sv.ptr, C.int(sv.len))
		probabilities[className] = float32(cClassProbability)
	}
	var cFeatureContributionsIter *C.tangram_multiclass_classification_predict_output_feature_contributions_iter
	C.tangram_multiclass_classification_predict_output_get_feature_contributions_iter(cOutput, &cFeatureContributionsIter)
	defer C.tangram_multiclass_classification_predict_output_feature_contributions_iter_delete(cFeatureContributionsIter)
	featureContributions := make(map[string]FeatureContributions)
	if cFeatureContributionsIter != nil {
		var cFeatureContributions *C.tangram_feature_contributions
		for C.tangram_multiclass_classification_predict_output_feature_contributions_iter_next(cFeatureContributionsIter, &sv, &cFeatureContributions) {
			className := C.GoStringN(sv.ptr, C.int(sv.len))
			featureContributions[className] = makeFeatureContributions(cFeatureContributions)
		}
	}

	return MulticlassClassificationPredictOutput{
		ClassName:            predictedClassName,
		Probability:          float32(cProbability),
		Probabilities:        probabilities,
		FeatureContributions: featureContributions,
	}
}

// FeatureContributions returns the FeatureContributions from a RegressionPredictOutput.
func makeFeatureContributions(cfcs *C.tangram_feature_contributions) FeatureContributions {
	var len C.size_t
	C.tangram_feature_contributions_get_entries_len(cfcs, &len)
	featureContributions := make([]FeatureContributionEntry, int(len))
	var baseline C.float
	C.tangram_feature_contributions_get_baseline_value(cfcs, &baseline)
	var output C.float
	C.tangram_feature_contributions_get_output_value(cfcs, &output)
	var cFeatureContribution *C.tangram_feature_contribution_entry
	for i := range featureContributions {
		C.tangram_feature_contributions_get_entry_at_index(cfcs, C.size_t(i), &cFeatureContribution)
		featureContributions[i] = makeFeatureContribution(cFeatureContribution)
	}
	return FeatureContributions{
		BaselineValue: float32(baseline),
		OutputValue:   float32(output),
		Entries:       featureContributions,
	}
}

func makeFeatureContribution(f *C.tangram_feature_contribution_entry) FeatureContributionEntry {
	var cType C.tangram_feature_contribution_entry_type

	C.tangram_feature_contribution_entry_get_type(f, &cType)
	switch cType {
	case IdentityFeatureContributionType:
		return makeIdentityFeatureContribution(f)
	case NormalizedFeatureContributionType:
		return makeNormalizedFeatureContribution(f)
	case OneHotEncodedFeatureContributionType:
		return makeOneHotEncodedFeatureContribution(f)
	case BagOfWordsFeatureContributionType:
		return makeBagOfWordsFeatureContribution(f)
	case BagOfWordsCosineSimilarityFeatureContributionType:
		return makeBagOfWordsCosineSimilarityFeatureContribution(f)
	case WordEmbeddingFeatureContributionType:
		return makeWordEmbeddingFeatureContribution(f)
	}
	return nil
}

func makeIdentityFeatureContribution(f *C.tangram_feature_contribution_entry) IdentityFeatureContribution {
	var cFeatureContribution *C.tangram_identity_feature_contribution
	var cColumnName C.tangram_string_view
	var cFeatureValue C.float
	var cFeatureContributionValue C.float
	C.tangram_feature_contribution_entry_as_identity(f, &cFeatureContribution)
	C.tangram_identity_feature_contribution_get_column_name(cFeatureContribution, &cColumnName)
	C.tangram_identity_feature_contribution_get_feature_value(cFeatureContribution, &cFeatureValue)
	C.tangram_identity_feature_contribution_get_feature_contribution_value(cFeatureContribution, &cFeatureContributionValue)
	return IdentityFeatureContribution{
		ColumnName:               C.GoStringN(cColumnName.ptr, C.int(cColumnName.len)),
		FeatureContributionValue: float32(cFeatureContributionValue),
		FeatureValue:             float32(cFeatureValue),
	}
}

func makeNormalizedFeatureContribution(f *C.tangram_feature_contribution_entry) NormalizedFeatureContribution {
	var cFeatureContribution *C.tangram_normalized_feature_contribution
	var cColumnName C.tangram_string_view
	var featureValue C.float
	var featureContributionValue C.float
	C.tangram_feature_contribution_entry_as_normalized(f, &cFeatureContribution)
	C.tangram_normalized_feature_contribution_get_column_name(cFeatureContribution, &cColumnName)
	C.tangram_normalized_feature_contribution_get_feature_value(cFeatureContribution, &featureValue)
	C.tangram_normalized_feature_contribution_get_feature_contribution_value(cFeatureContribution, &featureContributionValue)
	return NormalizedFeatureContribution{
		ColumnName:               C.GoStringN(cColumnName.ptr, C.int(cColumnName.len)),
		FeatureValue:             float32(featureValue),
		FeatureContributionValue: float32(featureContributionValue),
	}
}

func makeOneHotEncodedFeatureContribution(f *C.tangram_feature_contribution_entry) OneHotEncodedFeatureContribution {
	var cFeatureContribution *C.tangram_one_hot_encoded_feature_contribution
	var cColumnName C.tangram_string_view
	var cVariant C.tangram_string_view
	var cFeatureValue C.bool
	var cFeatureContributionValue C.float
	C.tangram_feature_contribution_entry_as_one_hot_encoded(f, &cFeatureContribution)
	C.tangram_one_hot_encoded_feature_contribution_get_column_name(cFeatureContribution, &cColumnName)
	C.tangram_one_hot_encoded_feature_contribution_get_variant(cFeatureContribution, &cVariant)
	C.tangram_one_hot_encoded_feature_contribution_get_feature_value(cFeatureContribution, &cFeatureValue)
	C.tangram_one_hot_encoded_feature_contribution_get_feature_contribution_value(cFeatureContribution, &cFeatureContributionValue)
	return OneHotEncodedFeatureContribution{
		ColumnName:               C.GoStringN(cColumnName.ptr, C.int(cColumnName.len)),
		Variant:                   C.GoStringN(cVariant.ptr, C.int(cVariant.len)),
		FeatureContributionValue: float32(cFeatureContributionValue),
		FeatureValue:             bool(cFeatureValue),
	}
}

func makeBagOfWordsFeatureContribution(f *C.tangram_feature_contribution_entry) BagOfWordsFeatureContribution {
	var cFeatureContribution *C.tangram_bag_of_words_feature_contribution
	var cColumnName C.tangram_string_view
	var cNGram *C.tangram_ngram
	var cFeatureValue C.float
	var cFeatureContributionValue C.float
	C.tangram_feature_contribution_entry_as_bag_of_words(f, &cFeatureContribution)
	C.tangram_bag_of_words_feature_contribution_get_column_name(cFeatureContribution, &cColumnName)
	C.tangram_bag_of_words_feature_contribution_get_feature_contribution_value(cFeatureContribution, &cFeatureContributionValue)
	C.tangram_bag_of_words_feature_contribution_get_ngram(cFeatureContribution, &cNGram)
	C.tangram_bag_of_words_feature_contribution_get_feature_value(cFeatureContribution, &cFeatureValue)
	return BagOfWordsFeatureContribution{
		ColumnName:               C.GoStringN(cColumnName.ptr, C.int(cColumnName.len)),
		NGram:                    makeNGram(cNGram),
		FeatureContributionValue: float32(cFeatureContributionValue),
		FeatureValue:             float32(cFeatureValue),
	}
}

const (
	UnigramType = iota
	BigramType
)

func makeNGram(n *C.tangram_ngram) NGram {
	var cType C.tangram_ngram_type
	C.tangram_ngram_get_type(n, &cType)
	switch cType {
	case UnigramType:
		return makeUnigramNGram(n)
	case BigramType:
		return makeBigramNGram(n)
	}
	return nil
}

func makeUnigramNGram(n *C.tangram_ngram) Unigram {
	var cToken C.tangram_string_view
	C.tangram_unigram_get_token(n, &cToken)
	return Unigram{
		Token: C.GoStringN(cToken.ptr, C.int(cToken.len)),
	}
}

func makeBigramNGram(n *C.tangram_ngram) Bigram {
	var cTokenA C.tangram_string_view
	var cTokenB C.tangram_string_view
	C.tangram_bigram_get_token_a(n, &cTokenA)
	C.tangram_bigram_get_token_b(n, &cTokenB)
	return Bigram{
		TokenA: C.GoStringN(cTokenA.ptr, C.int(cTokenA.len)),
		TokenB: C.GoStringN(cTokenB.ptr, C.int(cTokenB.len)),
	}
}

func makeBagOfWordsCosineSimilarityFeatureContribution(f *C.tangram_feature_contribution_entry) BagOfWordsCosineSimilarityFeatureContribution {
	var cFeatureContribution *C.tangram_bag_of_words_cosine_similarity_feature_contribution
	var cColumnNameA C.tangram_string_view
	var cColumnNameB C.tangram_string_view
	var cFeatureValue C.float
	var cFeatureContributionValue C.float
	C.tangram_feature_contribution_entry_as_bag_of_words_cosine_similarity(f, &cFeatureContribution)
	C.tangram_bag_of_words_cosine_similarity_feature_contribution_get_column_name_a(cFeatureContribution, &cColumnNameA)
	C.tangram_bag_of_words_cosine_similarity_feature_contribution_get_column_name_b(cFeatureContribution, &cColumnNameB)
	C.tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_contribution_value(cFeatureContribution, &cFeatureContributionValue)
	C.tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_value(cFeatureContribution, &cFeatureValue)
	return BagOfWordsCosineSimilarityFeatureContribution{
		ColumnNameA:               C.GoStringN(cColumnNameA.ptr, C.int(cColumnNameA.len)),
		ColumnNameB:               C.GoStringN(cColumnNameB.ptr, C.int(cColumnNameB.len)),
		FeatureContributionValue: float32(cFeatureContributionValue),
		FeatureValue:             float32(cFeatureValue),
	}
}

func makeWordEmbeddingFeatureContribution(f *C.tangram_feature_contribution_entry) WordEmbeddingFeatureContribution {
	var cFeatureContribution *C.tangram_word_embedding_feature_contribution
	var cColumnName C.tangram_string_view
	var cValueIndex C.size_t
	var cFeatureContributionValue C.float
	C.tangram_feature_contribution_entry_as_word_embedding(f, &cFeatureContribution)
	C.tangram_word_embedding_feature_contribution_get_column_name(cFeatureContribution, &cColumnName)
	C.tangram_word_embedding_feature_contribution_get_feature_contribution_value(cFeatureContribution, &cFeatureContributionValue)
	C.tangram_word_embedding_feature_contribution_get_value_index(cFeatureContribution, &cValueIndex)
	return WordEmbeddingFeatureContribution{
		ColumnName:               C.GoStringN(cColumnName.ptr, C.int(cColumnName.len)),
		ValueIndex:               int(cValueIndex),
		FeatureContributionValue: float32(cFeatureContributionValue),
	}
}

// Send a prediction event to the app. If you want to batch events, you can use `model.EnqueueLogPrediction` instead.
func (m Model) LogPrediction(args LogPredictionArgs) error {
	return m.logEvent(m.predictionEvent(args))
}

// Add a prediction event to the queue. Remember to call `model.FlushLogQueue` at a later point to send the event to the app.
func (m Model) EnqueueLogPrediction(args LogPredictionArgs) {
	m.logQueue = append(m.logQueue, m.predictionEvent(args))
}

//  Send a true value event to the app. If you want to batch events, you can use `model.EnqueueLogTrueValue` instead.
func (m Model) LogTrueValue(args LogTrueValueArgs) error {
	return m.logEvent(m.trueValueEvent(args))
}

// Add a true value event to the queue. Remember to call `model.FlushLogQueue` at a later point to send the event to the app.
func (m Model) EnqueueLogTrueValue(args LogTrueValueArgs) {
	m.logQueue = append(m.logQueue, m.trueValueEvent(args))
}

// Send all events in the queue to the app.
func (m Model) FlushLogQueue() error {
	err := m.logEvents(m.logQueue)
	if err != nil {
		return err
	}
	m.logQueue = []event{}
	return nil
}

func (m Model) logEvent(e event) error {
	return m.logEvents([]event{e})
}

func (m Model) logEvents(events []event) error {
	body, err := json.Marshal(events)
	if err != nil {
		return err
	}
	req, err := http.NewRequest(
		"POST",
		m.options.TangramURL+"/track",
		bytes.NewReader(body),
	)
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()
	if res.StatusCode < 200 || res.StatusCode > 299 {
		body, err := ioutil.ReadAll(res.Body)
		if err != nil {
			return err
		}
		return errors.New(string(body))
	}
	return nil
}

func (m Model) predictionEvent(args LogPredictionArgs) event {
	return event{
		"date":       time.Now().Format(time.RFC3339),
		"identifier": args.Identifier,
		"input":      args.Input,
		"modelId":    m.ID(),
		"options":    args.Options,
		"output":     args.Output,
		"type":       "prediction",
	}
}

func (m Model) trueValueEvent(args LogTrueValueArgs) event {
	return event{
		"date":       time.Now().Format(time.RFC3339),
		"identifier": args.Identifier,
		"modelId":    m.ID(),
		"trueValue":  args.TrueValue,
		"type":       "true_value",
	}
}
