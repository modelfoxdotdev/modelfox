require 'date'
require 'ffi'
require 'json'
require 'net/http'
require 'rbconfig'

# This is the main module in the `tangram` gem.
module Tangram

  # These are the options passed when loading a model.
  class LoadModelOptions
    # If you are running the app locally or on your own server, use this field to provide the url to it. If not specified, the default value is https://app.tangram.xyz.
    attr_reader :tangram_url
    def initialize(tangram_url:)
      @tangram_url = tangram_url
    end
  end

  # These are the options passed to `predict`.
  class PredictOptions
    # If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.
    attr_reader :threshold
    # Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `feature_contributions` field of the predict output.
    attr_reader :compute_feature_contributions
    def initialize(compute_feature_contributions:, threshold: nil)
      @threshold = threshold
      @compute_feature_contributions = compute_feature_contributions
    end
    def to_json(*args)
      {'threshold' => @threshold, 'compute_feature_contributions' => @compute_feature_contributions}.to_json(*args)
    end
  end

  # `predict` outputs `RegressionPredictOutput` when the model's task is regression.
  class RegressionPredictOutput
    # This is the predicted value.
    attr_reader :value
    # If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
    attr_reader :feature_contributions
    def initialize(value:, feature_contributions:)
      @value = value
      @feature_contributions = feature_contributions
    end
    def to_json(*args)
      {'value' => @value}.to_json(*args)
    end
  end

  # `predict` outputs `BinaryClassificationPredictOutput` when the model's task is binary classification.
  class BinaryClassificationPredictOutput
    # This is the name of the predicted class.
    attr_reader :class_name
    # This is the probability the model assigned to the predicted class.
    attr_reader :probability
    # If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
    attr_reader :feature_contributions
    def initialize(class_name:, probability:, feature_contributions:)
      @class_name = class_name
      @probability = probability
      @feature_contributions = feature_contributions
    end
    def to_json(*args)
      {'class_name' => @class_name, "probability" => @probability}.to_json(*args)
    end
  end

  # `predict` outputs `MulticlassClassificationPredictOutput` when the model's task is multiclass classification.
  class MulticlassClassificationPredictOutput
    # This is the name of the predicted class.
    attr_reader :class_name
    # This is the probability the model assigned to the predicted class.
    attr_reader :probability
    # This value maps from class names to the probability the model assigned to each class.
    attr_reader :probabilities
    # If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `output_value` is the predicted class.
    attr_reader :feature_contributions
    def initialize(class_name:, probability:, probabilities:, feature_contributions:)
      @class_name = class_name
      @probability = probability
      @probabilities = probabilities
      @feature_contributions = feature_contributions
    end
    def to_json(*args)
      {'class_name' => @class_name, "probability" => @probability, "probabilities" => @probabilities}.to_json(*args)
    end
  end

  # This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.
  class FeatureContributions
    # This is the value the model would output if all features had baseline values.
    attr_reader :baseline
    # This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.
    attr_reader :output
    # This list will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
    attr_reader :entries
    def initialize(baseline:, output:, entries:)
      @baseline = baseline
      @output = output
      @entries = entries
    end
  end

  # This describes the contribution of a feature from an identity feature group.
  class IdentityFeatureContribution
    # This is the name of the source column for the feature group.
    attr_reader :column_name
    # This is the value of the feature.
    attr_reader :feature_value
    # This is the amount that the feature contributed to the output.
    attr_reader :feature_contribution_value
    def initialize(column_name:, feature_contribution_value:, feature_value:)
      @column_name = column_name
      @feature_value = feature_value
      @feature_contribution_value = feature_contribution_value
    end
  end

  # This describes the contribution of a feature from a normalized feature group.
  class NormalizedFeatureContribution
    # This is the name of the source column for the feature group.
    attr_reader :column_name
    # This is the value of the feature.
    attr_reader :feature_value
    # This is the amount that the feature contributed to the output.
    attr_reader :feature_contribution_value
    def initialize(column_name:, feature_value:, feature_contribution_value:)
      @column_name = column_name
      @feature_value = feature_value
      @feature_contribution_value = feature_contribution_value
    end
  end

  # This describes the contribution of a feature from a one hot encoded feature group.
  class OneHotEncodedFeatureContribution
    # This is the name of the source column for the feature group.
    attr_reader :column_name
    # This is the enum variant the feature indicates the presence of.
    attr_reader :variant
    # This is the value of the feature.
    attr_reader :feature_value
    # This is the amount that the feature contributed to the output.b
    attr_reader :feature_contribution_value
    def initialize(column_name:, variant:, feature_contribution_value:, feature_value:)
      @column_name = column_name
      @variant = variant
      @feature_contribution_value = feature_contribution_value
      @feature_value = feature_value
    end
  end

  # This describes the contribution of a feature from a bag of words feature group.
  class BagOfWordsFeatureContribution
    # This is the name of the source column for the feature group.
    attr_reader :column_name
    # This is the ngram for the feature.
    attr_reader :ngram
    # This is the value of the feature.
    attr_reader :feature_value
    # This is the amount that the feature contributed to the output.
    attr_reader :feature_contribution_value
    def initialize(column_name:, ngram:, feature_contribution_value:, feature_value:)
      @column_name = column_name
      @ngram = ngram
      @feature_contribution_value = feature_contribution_value
      @feature_value = feature_value
    end
  end

  # This describes a unigram ngram.
  class Unigram
    # This is the token.
    attr_reader :token
    def initialize(token)
      @token = token
    end
  end

  # This describes a bigram ngram.
  class Bigram
    # This is the first token in the bigram.
    attr_reader :token_a
    # This is the second token in the bigram.
    attr_reader :token_b
    def initialize(token_a:, token_b:)
      @token_a = token_a
      @token_b = token_b
    end
  end

  # This describes the contribution of a feature from a bag of words cosine similarity feature group.
  class BagOfWordsCosineSimilarityFeatureContribution
    # This is the name of the source column a for the feature group.
    attr_reader :column_name_a
    # This is the name of the source column b for the feature group.
    attr_reader :column_name_b
    # This is the value of the feature.
    attr_reader :feature_value
    # This is the amount that the feature contributed to the output.
    attr_reader :feature_contribution_value
    def initialize(column_name_a:, column_name_b:, feature_contribution_value:, feature_value:)
      @column_name_a = column_name_a
      @column_name_b = column_name_b
      @feature_contribution_value = feature_contribution_value
      @feature_value = feature_value
    end
  end

  # This describes the contribution of a feature from a word embedding feature group.
  class WordEmbeddingFeatureContribution
    # This is the name of the source column for the feature group.
    attr_reader :column_name
    # This is the index of the feature in the word embedding.
    attr_reader :value_index
    # This is the amount that the feature contributed to the output.
    attr_reader :feature_contribution_value
    def initialize(column_name:, value_index:, feature_contribution_value:)
      @column_name = column_name
      @value_index = value_index
      @feature_contribution_value = feature_contribution_value
    end
  end

  # Use this class to load a model, make predictions, and log events to the app.
  class Model
    # Load a model from the `.tangram` file at `path`.
    # @param path [String] The path to the `.tangram` file.
    # @param options [LoadModelOptions] The options to use when loading the model.
    # @return [Model]
    def self.from_path(path, options: nil)
      c_model = FFI::MemoryPointer.new(:pointer)
      c_err = LibTangram.tangram_model_from_path(path, c_model)
      unless c_err.null?
        c_err = FFI::AutoPointer.new(c_err, LibTangram.method(:tangram_error_delete))
        c_error_s = LibTangram::TangramStringView.new
        LibTangram.tangram_error_get_message(c_err, c_error_s)
        raise c_error_s.into_string
      end
      new(c_model, options: options)
    end

    # Load a model from bytes instead of a file. You should use this only if you already have a `.tangram` loaded into memory. Otherwise, use `Model.from_path`, which is faster because it memory maps the file.
    # @param bytes [String] The bytes for the .tangram model.
    # @param options [LoadModelOptions] The options to use when loading the model.
    # @return [Model]
    def self.from_bytes(bytes, options: nil)
      c_model = FFI::MemoryPointer.new(:pointer)
      c_err = LibTangram.tangram_model_from_bytes(bytes, bytes.size, c_model)
      unless err.null?
        c_err = FFI::AutoPointer.new(c_err, LibTangram.method(:tangram_error_delete))
        c_error_s = LibTangram::TangramStringView.new
        LibTangram.tangram_error_get_message(c_err, c_error_s)
        raise errors.into_string
      end
      new(c_model, options: options)
    end

    def initialize(c_model, options: nil)
      @tangram_url = options&.tangram_url.nil? ? 'https://app.tangram.xyz' : options&.tangram_url
      @log_queue = []
      @model = FFI::AutoPointer.new(c_model.read_pointer, LibTangram.method(:tangram_model_delete))
    end

    # Retrieve the model's id.
    def id
      c_id = LibTangram::TangramStringView.new
      LibTangram.tangram_model_get_id(@model, c_id)
      c_id.into_string
    end

    # Make a prediction!
    # @param input [Array<Hash{String, Symbol => String, Number}>, Hash{String, Symbol => String, Number}] A predict input is either a single predict input which is a map from symbols or strings to strings or floats or an array of such maps. The keys should match the columns in the CSV file you trained your model with.
    # @param options [PredictOptions] These are the predict options.
    # @return [Array<RegressionPredictOutput, BinaryClassificationPredictOutput, MulticlassClassificationPredictOutput>, RegressionPredictOutput, BinaryClassificationPredictOutput, MulticlassClassificationPredictOutput]. Return a single output if `input` was a single input, or an array if `input` was an array of `input`s.
    def predict(input, options: nil)
      is_array = input.is_a?(Array)
      input = is_array ? input : [input]
      c_input_vec = new_predict_input_vec(input)
      c_options = new_predict_options(options)
      c_output_vec = FFI::MemoryPointer.new(:pointer)
      c_error = LibTangram.tangram_model_predict(@model, c_input_vec, c_options, c_output_vec)
      raise 'tangram error' unless c_error.null?
      c_output_vec = FFI::AutoPointer.new(c_output_vec.read_pointer, LibTangram.method(:tangram_predict_output_vec_delete))
      output = predict_output_vec_from_tangram_predict_output_vec(c_output_vec)
      is_array ? output : output[0]
    end

    # Send a prediction event to the app. If you want to batch events, you can use `enqueue_log_prediction` instead.
    # @param identifier [String, Number] This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
    # @param input [Hash{String, Symbol => String, Number}] A single `PredictInput`.
    # @param output [PredictOutput] A single `PredictOutput`.
    # @param options [PredictOptions] This is the same `predictOptions` value that you passed to `predict`.
    def log_prediction(identifier:, input:, output:, options: nil)
      event = prediction_event(
        identifier: identifier,
        input: input,
        output: output,
        options: options
      )
      log_event(event)
    end

    # Add a prediction event to the queue. Remember to call `flush_log_queue` at a later point to send the event to the app.
    # @param identifier [String, Number] This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
    # @param input [Hash{String, Symbol => String, Number}] A single `PredictInput`.
    # @param output [PredictOutput] A single `PredictOutput`.
    # @param options [PredictOptions] This is the same `predictOptions` value that you passed to `predict`.
    def enqueue_log_prediction(identifier:, input:, output:, options: nil)
      event = prediction_event(
        identifier: identifier,
        input: input,
        output: output,
        options: options
      )
      log_queue.push(event)
    end

    # Send a true value event to the app. If you want to batch events, you can use `enqueue_log_true_value` instead.
    # @param identifier [String, Number] This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
    # @param true_value [String, Number] This is the true value for the prediction.
    def log_true_value(identifier:, true_value:)
      event = true_value_event(
        identifier: identifier,
        true_value: true_value
      )
      log_event(event)
    end

    # Add a true value event to the queue. Remember to call `flush_log_queue` at a later point to send the event to the app.
    # @param identifier [String, Number] This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
    # @param true_value [String, Number] This is the true value for the prediction.
    def enqueue_log_true_value(identifier:, true_value:)
      event = true_value_event(
        identifier: identifier,
        true_value: true_value
      )
      log_queue.push(event)
    end

    # Send all events in the queue to the app.
    def flush_log_queue
      log_events(@log_queue)
      @log_queue = []
    end

    private

    def log_event(event)
      log_events([event])
    end

    def log_events(events)
      headers = {
        'Content-Type': 'application/json'
      }
      uri = URI("#{@tangram_url}/track")
      http = Net::HTTP.new(uri.host, uri.port)
      request = Net::HTTP::Post.new(uri.request_uri, headers)
      request.body = events.to_json
      response = http.request(request)
      raise response unless response.is_a? Net::HTTPSuccess
    end

    def prediction_event(identifier:, input:, output:, options: nil)
      {
        date: DateTime.now.rfc3339,
        identifier: identifier,
        input: input,
        model_id: id,
        options: options,
        output: output,
        type: 'prediction'
      }
    end

    def true_value_event(identifier:, true_value:)
      {
        date: DateTime.now.rfc3339,
        identifier: identifier,
        model_id: id,
        true_value: true_value,
        type: 'true_value'
      }
    end

    def new_predict_options(options)
      c_options = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_options_new(c_options)
      c_options = FFI::AutoPointer.new(c_options.read_pointer, LibTangram.method(:tangram_predict_options_delete))
      unless options.nil?
        unless options.threshold.nil?
          LibTangram.tangram_predict_options_set_threshold(c_options, options.threshold)
        end
        unless options.compute_feature_contributions.nil?
          LibTangram.tangram_predict_options_set_compute_feature_contributions(c_options, options.compute_feature_contributions)
        end
      end
      c_options
    end

    def new_predict_input_vec(input_vec)
      c_inputs = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_input_vec_new(c_inputs)
      c_inputs = FFI::AutoPointer.new(c_inputs.read_pointer, LibTangram.method(:tangram_predict_input_vec_delete))
      (0...input_vec.length).each do |input_index|
        input = input_vec[input_index]
        predict_input = new_predict_input(input)
        LibTangram.tangram_predict_input_vec_push(c_inputs, predict_input)
      end
      c_inputs
    end

    def new_predict_input(input)
      c_input = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_input_new(c_input)
      c_input = c_input.read_pointer
      input.each do |key, value|
        is_float = value.is_a?(Float)
        is_string = value.is_a?(String)
        if is_float
          LibTangram.tangram_predict_input_set_value_number(c_input, key.to_s, value)
        elsif is_string
          LibTangram.tangram_predict_input_set_value_string(c_input, key.to_s, value)
        else
          raise 'value for key %s is not a float or a string' % key
        end
      end
      c_input
    end

    def predict_output_vec_from_tangram_predict_output_vec(c_output_vec)
      outputs = []
      c_output_len = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_predict_output_vec_len(c_output_vec, c_output_len)
      output_len = c_output_len.read(:int)
      (0...output_len).each do |output_index|
        c_output = FFI::MemoryPointer.new(:pointer)
        LibTangram.tangram_predict_output_vec_get_at_index(c_output_vec, output_index, c_output)
        c_output = c_output.read_pointer
        outputs.push(predict_output_from_tangram_predict_output(c_output))
      end

      outputs
    end

    def predict_output_from_tangram_predict_output(c_output)
      c_task_type = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_model_get_task(@model, c_task_type)
      case c_task_type.read(:int)
      when LibTangram::TangramTaskType[:regression]
        regression_output_from_tangram_predict_output(c_output)
      when LibTangram::TangramTaskType[:binary_classification]
        binary_classification_output_from_tangram_predict_output(c_output)
      when LibTangram::TangramTaskType[:multiclass_classification]
        multiclass_classification_output_from_tangram_predict_output(c_output)
      end
    end

    def regression_output_from_tangram_predict_output(c_output)
      c_regression_output = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_output_as_regression(c_output, c_regression_output)
      c_regression_output = c_regression_output.read_pointer
      c_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_regression_predict_output_get_value(c_regression_output, c_value)
      value = c_value.read(:float)
      c_feature_contributions = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_regression_predict_output_get_feature_contributions(c_regression_output, c_feature_contributions)
      c_feature_contributions = c_feature_contributions.read_pointer
      feature_contributions = c_feature_contributions.null? ? {} : get_feature_contributions(c_feature_contributions)
      RegressionPredictOutput.new(
        value: value,
        feature_contributions: feature_contributions
      )
    end

    def binary_classification_output_from_tangram_predict_output(c_output)
      c_binary_classification_output = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_output_as_binary_classification(c_output, c_binary_classification_output)
      c_binary_classification_output = c_binary_classification_output.read_pointer
      c_probability = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_binary_classification_predict_output_get_probability(c_binary_classification_output, c_probability)
      probability = c_probability.read(:float)
      c_class_name = LibTangram::TangramStringView.new
      LibTangram.tangram_binary_classification_predict_output_get_class_name(c_binary_classification_output, c_class_name)
      class_name = c_class_name.into_string
      c_feature_contributions = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_binary_classification_predict_output_get_feature_contributions(c_binary_classification_output, c_feature_contributions)
      c_feature_contributions = c_feature_contributions.read_pointer
      feature_contributions = c_feature_contributions.null? ? {} : get_feature_contributions(c_feature_contributions)
      BinaryClassificationPredictOutput.new(
        class_name: class_name,
        probability: probability,
        feature_contributions: feature_contributions
      )
    end

    def multiclass_classification_output_from_tangram_predict_output(c_output)
      c_multiclass_classification_output = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_predict_output_as_multiclass_classification(c_output, c_multiclass_classification_output)
      c_multiclass_classification_output = c_multiclass_classification_output.read_pointer
      c_probability = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_multiclass_classification_predict_output_get_probability(c_multiclass_classification_output, c_probability)
      probability = c_probability.read(:float)
      c_class_name = LibTangram::TangramStringView.new
      LibTangram.tangram_multiclass_classification_predict_output_get_class_name(c_multiclass_classification_output, c_class_name)
      class_name = c_class_name.into_string
      probabilities = multiclass_classification_output_get_probabilities(c_multiclass_classification_output)
      feature_contributions = multiclass_classification_output_get_feature_contributions(c_multiclass_classification_output)
      MulticlassClassificationPredictOutput.new(
        class_name: class_name,
        probability: probability,
        probabilities: probabilities,
        feature_contributions: feature_contributions
      )
    end

    def multiclass_classification_output_get_probabilities(c_multiclass_classification_output)
      probabilities = {}
      c_probabilities_iter = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_multiclass_classification_predict_output_get_probabilities_iter(c_multiclass_classification_output, c_probabilities_iter)
      c_probabilities_iter = FFI::AutoPointer.new(c_probabilities_iter.read_pointer, LibTangram.method(:tangram_multiclass_classification_predict_output_probabilities_iter_delete))
      c_class_probability = FFI::MemoryPointer.new(:float)
      c_class_name = LibTangram::TangramStringView.new
      while LibTangram.tangram_multiclass_classification_predict_output_probabilities_iter_next(c_probabilities_iter, c_class_name, c_class_probability)
        class_name = c_class_name.into_string
        probabilities[class_name] = c_class_probability.read(:float)
      end
      probabilities
    end

    def multiclass_classification_output_get_feature_contributions(c_multiclass_classification_output)
      c_feature_contributions_iter = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_multiclass_classification_predict_output_get_feature_contributions_iter(c_multiclass_classification_output, c_feature_contributions_iter)
      c_feature_contributions_iter = FFI::AutoPointer.new(c_feature_contributions_iter.read_pointer, LibTangram.method(:tangram_multiclass_classification_predict_output_feature_contributions_iter_delete))
      feature_contributions = {}
      unless c_feature_contributions_iter.null?
        c_class_name = LibTangram::TangramStringView.new
        c_feature_contributions_ptr = FFI::MemoryPointer.new(:pointer)
        while LibTangram.tangram_multiclass_classification_predict_output_feature_contributions_iter_next(c_feature_contributions_iter, c_class_name, c_feature_contributions_ptr)
          class_name = c_class_name.into_string
          c_feature_contributions = c_feature_contributions_ptr.read_pointer
          unless c_feature_contributions.null?
            feature_contributions[class_name] = get_feature_contributions(c_feature_contributions)
          end
        end
      end
      feature_contributions
    end

    def get_feature_contributions(c_feature_contributions)
      c_baseline = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_feature_contributions_get_baseline_value(c_feature_contributions, c_baseline)
      baseline = c_baseline.read(:float)
      c_output = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_feature_contributions_get_output_value(c_feature_contributions, c_output)
      output = c_output.read(:float)
      feature_contribution_entries = get_feature_contributions_entries(c_feature_contributions)
      FeatureContributions.new(
        baseline: baseline,
        output: output,
        entries: feature_contribution_entries
      )
    end

    def get_feature_contributions_entries(c_feature_contributions)
      c_len = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_feature_contributions_get_entries_len(c_feature_contributions, c_len)
      len = c_len.read(:int)
      feature_contributions = []
      (0...len).each do |i|
        c_feature_contribution = FFI::MemoryPointer.new(:pointer)
        LibTangram.tangram_feature_contributions_get_entry_at_index(c_feature_contributions, i, c_feature_contribution)
        c_feature_contribution = c_feature_contribution.read_pointer
        feature_contributions.push(get_feature_contribution(c_feature_contribution))
      end
      feature_contributions
    end

    def get_feature_contribution(c_feature_contribution)
      c_feature_contribution_type = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_feature_contribution_entry_get_type(c_feature_contribution, c_feature_contribution_type)
      case c_feature_contribution_type.read(:int)
      when LibTangram::TangramFeatureContributionEntryType[:identity]
        get_identity_feature_contribution(c_feature_contribution)
      when LibTangram::TangramFeatureContributionEntryType[:normalized]
        get_normalized_feature_contribution(c_feature_contribution)
      when LibTangram::TangramFeatureContributionEntryType[:one_hot_encoded]
        get_one_hot_encoded_feature_contribution(c_feature_contribution)
      when LibTangram::TangramFeatureContributionEntryType[:bag_of_words]
        get_bag_of_words_feature_contribution(c_feature_contribution)
      when LibTangram::TangramFeatureContributionEntryType[:bag_of_words_cosine_similarity]
        get_bag_of_words_cosine_similarity_feature_contribution(c_feature_contribution)
      when LibTangram::TangramFeatureContributionEntryType[:word_embedding]
        get_word_embedding_feature_contribution(c_feature_contribution)
      end
    end

    def get_identity_feature_contribution(c_feature_contribution)
      c_identity_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_identity(c_feature_contribution, c_identity_feature_contribution)
      c_identity_feature_contribution = c_identity_feature_contribution.read_pointer
      c_column_name = LibTangram::TangramStringView.new
      LibTangram.tangram_identity_feature_contribution_get_column_name(c_identity_feature_contribution, c_column_name)
      column_name = c_column_name.into_string
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_identity_feature_contribution_get_feature_contribution_value(c_identity_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      c_feature_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_identity_feature_contribution_get_feature_value(c_identity_feature_contribution, c_feature_value)
      feature_value = c_feature_value.read(:float)
      IdentityFeatureContribution.new(
        column_name: column_name,
        feature_value: feature_value,
        feature_contribution_value: feature_contribution_value
      )
    end

    def get_normalized_feature_contribution(c_feature_contribution)
      c_normalized_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_normalized(
        c_feature_contribution, c_normalized_feature_contribution
      )
      c_normalized_feature_contribution = c_normalized_feature_contribution.read_pointer
      c_column_name = LibTangram::TangramStringView.new
      LibTangram.tangram_normalized_feature_contribution_get_column_name(c_normalized_feature_contribution, c_column_name)
      column_name = c_column_name.into_string
      c_feature_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_normalized_feature_contribution_get_feature_value(c_normalized_feature_contribution, c_feature_value)
      feature_value = c_feature_value.read(:float)
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_normalized_feature_contribution_get_feature_contribution_value(c_normalized_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      NormalizedFeatureContribution.new(
        column_name: column_name,
        feature_value: feature_value,
        feature_contribution_value: feature_contribution_value
      )
    end

    def get_one_hot_encoded_feature_contribution(c_feature_contribution)
      c_one_hot_encoded_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_one_hot_encoded(c_feature_contribution, c_one_hot_encoded_feature_contribution)
      c_one_hot_encoded_feature_contribution = c_one_hot_encoded_feature_contribution.read_pointer
      c_column_name = LibTangram::TangramStringView.new
      LibTangram.tangram_one_hot_encoded_feature_contribution_get_column_name(c_one_hot_encoded_feature_contribution, c_column_name)
      column_name = c_column_name.into_string
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_one_hot_encoded_feature_contribution_get_feature_contribution_value(c_one_hot_encoded_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      c_feature_value = FFI::MemoryPointer.new(:bool)
      LibTangram.tangram_one_hot_encoded_feature_contribution_get_feature_value(c_one_hot_encoded_feature_contribution, c_feature_value)
      feature_value = c_feature_value.read(:bool)
      c_variant = LibTangram::TangramStringView.new
      LibTangram.tangram_one_hot_encoded_feature_contribution_get_variant(c_one_hot_encoded_feature_contribution, c_variant)
      variant = c_variant[:ptr].null? ? nil : c_variant.into_string
      OneHotEncodedFeatureContribution.new(
        column_name: column_name,
        variant: variant,
        feature_contribution_value: feature_contribution_value,
        feature_value: feature_value
      )
    end

    def get_bag_of_words_feature_contribution(c_feature_contribution)
      c_bag_of_words_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_bag_of_words(c_feature_contribution, c_bag_of_words_feature_contribution)
      c_bag_of_words_feature_contribution = c_bag_of_words_feature_contribution.read_pointer
      c_column_name = LibTangram::TangramStringView.new
      LibTangram.tangram_bag_of_words_feature_contribution_get_column_name(c_bag_of_words_feature_contribution, c_column_name)
      column_name = c_column_name.into_string
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_bag_of_words_feature_contribution_get_feature_contribution_value(c_bag_of_words_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      c_feature_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_bag_of_words_feature_contribution_get_feature_value(c_bag_of_words_feature_contribution, c_feature_value)
      feature_value = c_feature_value.read(:float)
      c_ngram = FFI::MemoryPointer(:pointer)
      LibTangram.tangram_bag_of_words_feature_contribution_get_ngram(c_bag_of_words_feature_contribution, c_ngram)
      c_ngram = c_ngram.read_pointer
      ngram = get_ngram(c_ngram)
      BagOfWordsFeatureContribution.new(
        column_name: column_name,
        ngram: ngram,
        feature_contribution_value: feature_contribution_value,
        feature_value: feature_value
      )
    end

    def get_bag_of_words_cosine_similarity_feature_contribution(c_feature_contribution)
      c_bag_of_words_cosine_similarity_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_bag_of_words_cosine_similarity(c_feature_contribution, c_bag_of_words_cosine_similarity_feature_contribution)
      c_bag_of_words_cosine_similarity_feature_contribution = c_bag_of_words_cosine_similarity_feature_contribution.read_pointer
      c_column_name_a = LibTangram::TangramStringView.new
      LibTangram.tangram_bag_of_words_feature_contribution_get_column_name_a(c_bag_of_words_cosine_similarity_feature_contribution, c_column_name_a)
      column_name_a = c_column_name_a.into_string
      c_column_name_b = LibTangram::TangramStringView.new
      LibTangram.tangram_bag_of_words_feature_contribution_get_column_name_b(c_bag_of_words_cosine_similarity_feature_contribution, c_column_name_b)
      column_name_b = c_column_name_b.into_string
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_contribution_value(c_bag_of_words_cosine_similarity_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      c_feature_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_value(c_bag_of_words_cosine_similarity_feature_contribution, c_feature_value)
      feature_value = c_feature_value.read(:float)
      BagOfWordsFeatureContribution.new(
        column_name_a: column_name_a,
        column_name_b: column_name_b,
        feature_contribution_value: feature_contribution_value,
        feature_value: feature_value
      )
    end

    def get_word_embedding_feature_contribution(c_feature_contribution)
      c_word_embedding_feature_contribution = FFI::MemoryPointer.new(:pointer)
      LibTangram.tangram_feature_contribution_entry_as_word_embedding(
        c_feature_contribution, c_word_embedding_feature_contribution
      )
      c_word_embedding_feature_contribution = c_word_embedding_feature_contribution.read_pointer
      c_column_name = LibTangram::TangramStringView.new
      LibTangram.tangram_word_embedding_feature_contribution_get_column_name(c_word_embedding_feature_contribution, c_column_name)
      column_name = c_column_name.into_string
      c_feature_contribution_value = FFI::MemoryPointer.new(:float)
      LibTangram.tangram_word_embedding_feature_contribution_get_feature_contribution_value(c_word_embedding_feature_contribution, c_feature_contribution_value)
      feature_contribution_value = c_feature_contribution_value.read(:float)
      c_value_index = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_word_embedding_feature_contribution_get_value_index(c_word_embedding_feature_contribution, c_value_index)
      value_index = c_value_index.read(:int)
      WordEmbeddingFeatureContribution.new(
        column_name: column_name,
        value_index: value_index,
        feature_contribution_value: feature_contribution_value
      )
    end

    def get_ngram(ngram)
      c_ngram_type = FFI::MemoryPointer.new(:int)
      LibTangram.tangram_ngram_get_type(ngram, ngram_type)
      case c_ngram_type.read(:int)
      when LibTangram::TangramNGramType[:unigram]
        get_unigram_ngram(ngram)
      when LibTangram::TangramNGramType[:bigram]
        get_bigram_ngram(ngram)
      end
    end

    def get_unigram_ngram(ngram)
      c_token = LibTangram::TangramStringView.new
      LibTangram.tangram_unigram_get_token(ngram, c_token)
      token = c_token.into_string
      Unigram.new(
        token: token
      )
    end

    def get_bigram_ngram(ngram)
      c_token_a = LibTangram::TangramStringView.new
      c_token_b = LibTangram::TangramStringView.new
      LibTangram.tangram_bigram_get_token_a(ngram, c_token_a)
      LibTangram.tangram_bigram_get_token_b(ngram, c_token_b)
      token_a = c_token_a.into_string
      token_b = c_token_b.into_string
      Bigram.new(
        token_a: token_a,
        token_b: token_b
      )
    end
  end

  module LibTangram
    cpu = RbConfig::CONFIG['host_cpu']
    os = RbConfig::CONFIG['host_os']
    if cpu == 'x86_64' && os =~ /linux/
      library_path = 'libtangram/x86_64-unknown-linux-gnu/libtangram.so'
    elsif cpu == 'aarch64' && os =~ /linux/
      library_path = 'libtangram/aarch64-unknown-linux-gnu/libtangram.so'
    elsif cpu == 'x86_64' && os =~ /darwin/
      library_path = 'libtangram/x86_64-apple-darwin/libtangram.dylib'
    elsif (cpu == 'arm' || cpu == 'arm64') && os =~ /darwin/
      library_path = 'libtangram/aarch64-apple-darwin/libtangram.dylib'
    elsif cpu == 'x86_64' && os =~ /mingw/
      library_path = 'libtangram/x86_64-pc-windows-msvc/tangram.dll'
    else
      raise 'Tangram for Ruby does not yet support your combination of CPU architecture and operating system. Open an issue at https://github.com/tangramxyz/tangram/issues/new or email us at help@tangram.xyz to complain.'
    end
    extend FFI::Library
    ffi_lib File.expand_path(library_path.to_s, __dir__)

    class TangramStringView < FFI::Struct
      layout :ptr, :pointer,
            :len, :int
      def into_string
        self[:ptr].read_string(self[:len]).force_encoding('utf-8')
      end
    end

    TangramTaskType = enum(
      :regression,
      :binary_classification,
      :multiclass_classification
    )

    TangramFeatureContributionEntryType = enum(
      :identity,
      :normalized,
      :one_hot_encoded,
      :bag_of_words,
      :bag_of_words_cosine_similarity,
      :word_embedding
    )

    TangramNGramType = enum(
      :unigram,
      :bigram
    )

    typedef :pointer, :tangram_error

    attach_function :tangram_error_get_message, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_error_delete, [:pointer], :void
    attach_function :tangram_version, [TangramStringView.by_ref], :void
    attach_function :tangram_model_from_bytes, [:pointer, :int, :pointer], :tangram_error
    attach_function :tangram_model_from_path, [:string, :pointer], :tangram_error
    attach_function :tangram_model_delete, [:pointer], :void
    attach_function :tangram_model_get_id, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_model_get_task, [:pointer, :pointer], :void
    attach_function :tangram_predict_input_new, [:pointer], :void
    attach_function :tangram_predict_input_delete, [:pointer], :void
    attach_function :tangram_predict_input_set_value_number, [:pointer, :string, :double], :int
    attach_function :tangram_predict_input_set_value_string, [:pointer, :string, :string], :int
    attach_function :tangram_predict_input_vec_new, [:pointer], :void
    attach_function :tangram_predict_input_vec_delete, [:pointer], :void
    attach_function :tangram_predict_input_vec_push, [:pointer, :pointer], :void
    attach_function :tangram_predict_options_new, [:pointer], :void
    attach_function :tangram_predict_options_delete, [:pointer], :void
    attach_function :tangram_predict_options_set_threshold, [:pointer, :float], :void
    attach_function :tangram_predict_options_set_compute_feature_contributions, [:pointer, :bool], :void
    attach_function :tangram_model_predict, [:pointer, :pointer, :pointer, :pointer], :tangram_error
    attach_function :tangram_predict_output_delete, [:pointer], :void
    attach_function :tangram_predict_output_vec_delete, [:pointer], :void
    attach_function :tangram_predict_output_vec_len, [:pointer, :pointer], :void
    attach_function :tangram_predict_output_vec_get_at_index, [:pointer, :int, :pointer], :void
    attach_function :tangram_predict_output_as_regression, [:pointer, :pointer], :void
    attach_function :tangram_predict_output_as_binary_classification, [:pointer, :pointer], :void
    attach_function :tangram_predict_output_as_multiclass_classification, [:pointer, :pointer], :void
    attach_function :tangram_regression_predict_output_get_value, [:pointer, :pointer], :void
    attach_function :tangram_regression_predict_output_get_feature_contributions, [:pointer, :pointer], :void
    attach_function :tangram_binary_classification_predict_output_get_class_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_binary_classification_predict_output_get_probability, [:pointer, :pointer], :void
    attach_function :tangram_binary_classification_predict_output_get_feature_contributions, [:pointer, :pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_get_class_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_multiclass_classification_predict_output_get_probability, [:pointer, :pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_get_probabilities_len, [:pointer, :pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_probabilities_iter_delete, [:pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_get_probabilities_iter, [:pointer, :pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_probabilities_iter_next, [:pointer, :pointer, :pointer], :bool
    attach_function :tangram_multiclass_classification_predict_output_feature_contributions_iter_delete, [:pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_get_feature_contributions_iter, [:pointer, :pointer], :void
    attach_function :tangram_multiclass_classification_predict_output_feature_contributions_iter_next, [:pointer, :pointer, :pointer], :bool
    attach_function :tangram_feature_contributions_get_baseline_value, [:pointer, :pointer], :void
    attach_function :tangram_feature_contributions_get_output_value, [:pointer, :pointer], :void
    attach_function :tangram_feature_contributions_get_entries_len, [:pointer, :pointer], :void
    attach_function :tangram_feature_contributions_get_entry_at_index, [:pointer, :int, :pointer], :void
    attach_function :tangram_feature_contribution_entry_get_type, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_identity, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_normalized, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_one_hot_encoded, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_bag_of_words, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_bag_of_words_cosine_similarity, [:pointer, :pointer], :void
    attach_function :tangram_feature_contribution_entry_as_word_embedding, [:pointer, :pointer], :void
    attach_function :tangram_identity_feature_contribution_get_column_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_identity_feature_contribution_get_feature_value, [:pointer, :pointer], :void
    attach_function :tangram_identity_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
    attach_function :tangram_normalized_feature_contribution_get_column_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_normalized_feature_contribution_get_feature_value, [:pointer, :pointer], :void
    attach_function :tangram_normalized_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
    attach_function :tangram_one_hot_encoded_feature_contribution_get_column_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_one_hot_encoded_feature_contribution_get_variant, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_one_hot_encoded_feature_contribution_get_feature_value, [:pointer, :pointer], :void
    attach_function :tangram_one_hot_encoded_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
    attach_function :tangram_bag_of_words_feature_contribution_get_column_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bag_of_words_feature_contribution_get_feature_value, [:pointer, :pointer], :void
    attach_function :tangram_bag_of_words_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
    attach_function :tangram_bag_of_words_feature_contribution_get_ngram, [:pointer, :pointer], :void
    attach_function :tangram_ngram_get_type, [:pointer, :pointer], :void
    attach_function :tangram_unigram_get_token, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bigram_get_token_a, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bigram_get_token_b, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bag_of_words_cosine_similarity_feature_contribution_get_column_name_a, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bag_of_words_cosine_similarity_feature_contribution_get_column_name_b, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_value, [:pointer, :pointer], :void
    attach_function :tangram_bag_of_words_cosine_similarity_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
    attach_function :tangram_word_embedding_feature_contribution_get_column_name, [:pointer, TangramStringView.by_ref], :void
    attach_function :tangram_word_embedding_feature_contribution_get_value_index, [:pointer, :pointer], :void
    attach_function :tangram_word_embedding_feature_contribution_get_feature_contribution_value, [:pointer, :pointer], :void
  end
end
