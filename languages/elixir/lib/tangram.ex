defmodule Tangram do
  @moduledoc """
  This is the main module in the `tangram` package.
  """

  defmodule Model do
    @moduledoc """
    Use this struct to load a model, make predictions, and log events to the app.
    """
    @type t :: %__MODULE__{
            model: reference,
            log_queue: [Tangram.event()],
            tangram_url: String.t()
          }
    defstruct [
      :model,
      :log_queue,
      :tangram_url
    ]
  end

  defmodule LoadModelOptions do
    @moduledoc """
    These are the options passed when loading a model.

    ## `tangram_url`
    If you are running the app locally or on your own server, use this field to provide the url to it. If not specified, the default value is https://app.tangram.dev.
    """
    @type t :: %__MODULE__{
            tangram_url: String.t()
          }
    defstruct [
      :tangram_url
    ]
  end

  @typedoc """
  This is the input type of `Tangram.predict`. A predict input is a map from atoms or strings to strings or floats. The keys should match the columns in the CSV file you trained your model with.
  """
  @type predict_input :: %{(atom | String.t()) => String.t() | float}

  defmodule PredictOptions do
    @moduledoc """
    These are the options passed to `Tangram.predict`.

    ## `threshold`
    If your model is a binary classifier, use this field to make predictions using the threshold you chose on the tuning page of the app. The default value is `0.5`.

    ## `compute_feature_contributions`
    Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `feature_contributions` field of the predict output.
    """
    @type t :: %__MODULE__{
            threshold: float,
            compute_feature_contributions: boolean
          }
    @derive Jason.Encoder
    defstruct [
      threshold: 0.5,
      compute_feature_contributions: false
    ]
  end

  @typedoc """
  This is the return type of `Tangram.predict`.
  """
  @type predict_output ::
          {:regression, RegressionPredictOutput.t()}
          | {:binary_classification, BinaryClassificationPredictOutput.t()}
          | {:multiclass_classification, MulticlassClassificationPredictOutput.t()}

  defmodule RegressionPredictOutput do
    @moduledoc """
    `Tangram.predict` outputs `{:regression, RegressionPredictOutput.t()}` when the model's task is regression.

    ## `value`
    This is the predicted value.

    ## `feature_contributions`
    If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
    """
    @type t :: %__MODULE__{
            value: float,
            feature_contributions: FeatureContributions.t() | nil
          }
    @derive {Jason.Encoder, except: [:feature_contributions]}
    defstruct [
      :value,
      :feature_contributions
    ]
  end

  defmodule BinaryClassificationPredictOutput do
    @moduledoc """
    `Tangram.predict` outputs `{:binary_classification, BinaryClassificationPredictOutput.t()}` when the model's task is binary classification.

    ## `class_name`
    This is the name of the predicted class.

    ## `probability`
    This is the probability the model assigned to the predicted class.

    ## `feature_contributions`
    If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
    """
    @type t :: %__MODULE__{
            class_name: String.t(),
            probability: float,
            feature_contributions: FeatureContributions.t() | nil
          }
    @derive {Jason.Encoder, except: [:feature_contributions]}
    defstruct [
      :class_name,
      :probability,
      :feature_contributions
    ]
  end

  defmodule MulticlassClassificationPredictOutput do
    @moduledoc """
    `Tangram.predict` outputs `{:multiclass_classification, MulticlassClassificationPredictOutput.t()}` when the model's task is multiclass classification.

    ## `class_name`
    This is the name of the predicted class.

    ## `probability`
    This is the probability the model assigned to the predicted class.

    ## `probabilities`
    This value maps from class names to the probability the model assigned to each class.

    ## `feature_contributions`
    If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `output_value` is the predicted class.
    """
    @type t :: %__MODULE__{
            class_name: String.t(),
            probability: float,
            probabilities: [float],
            feature_contributions: FeatureContributions.t() | nil
          }
    @derive {Jason.Encoder, except: [:feature_contributions]}
    defstruct [
      :class_name,
      :probability,
      :probabilities,
      :feature_contributions
    ]
  end

  defmodule FeatureContributions do
    @moduledoc """
    This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.

    ## `baseline_value`
    This is the value the model would output if all features had baseline values.

    ## `output_value`
    This is the value the model output. Any difference from the `baseline_value` is because of the deviation of the features from their baseline values.

    ## `entries`
    This list will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
    """
    @type t :: %__MODULE__{
            baseline_value: float,
            output_value: float,
            entries: [Tangram.feature_contribution_entry()]
          }
    defstruct [
      :baseline_value,
      :output_value,
      :entries
    ]
  end

  @typedoc """
  This identifies the type of a feature contribution.
  """
  @type feature_contribution_entry ::
          {:identity, IdentityFeatureContribution.t()}
          | {:normalized, NormalizedFeatureContribution.t()}
          | {:one_hot_encoded, OneHotEncodedFeatureContribution.t()}
          | {:bag_of_words, BagOfWordsFeatureContribution.t()}
          | {:bag_of_words_cosine_similarity, BagOfWordsCosineSimilarityFeatureContribution.t()}
          | {:word_embedding, WordEmbeddingFeatureContribution.t()}

  defmodule IdentityFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from an identity feature group.

    ## `column_name`
    This is the name of the source column for the identity feature group.

    ## `feature_value`
    This is the value of the feature.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type t :: %__MODULE__{
            column_name: String.t(),
            feature_value: float,
            feature_contribution_value: float
          }
    defstruct [
      :column_name,
      :feature_value,
      :feature_contribution_value
    ]
  end

  defmodule NormalizedFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from a normalized feature group.

    ## `column_name`
    This is the name of the source column for the normalized feature group.

    ## `feature_value`
    This is the value of the feature.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type t :: %__MODULE__{
            column_name: String.t(),
            feature_value: float,
            feature_contribution_value: float
          }
    defstruct [
      :column_name,
      :feature_value,
      :feature_contribution_value
    ]
  end

  defmodule OneHotEncodedFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from a one hot encoded feature group.

    ## `column_name`
    This is the name of the source column for the one hot encoded feature group.

    ## `variant`
    This is the enum variant the feature indicates the presence of.

    ## `feature_value`
    This is the value of the feature.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type t :: %__MODULE__{
            column_name: String.t(),
            variant: String.t(),
            feature_value: float,
            feature_contribution_value: float
          }
    defstruct [
      :column_name,
      :variant,
      :feature_value,
      :feature_contribution_value
    ]
  end

  defmodule BagOfWordsFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from a bag of words feature group.

    ## `column_name`
    This is the name of the source column for the bag of words feature group.

    ## `ngram`
    This is the ngram for the feature.

    ## `feature_value`
    This is the value of the feature.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type ngram :: String.t() | {String.t(), String.t()}
    @type t :: %__MODULE__{
            column_name: String.t(),
            ngram: ngram,
            feature_value: float,
            feature_contribution_value: float
          }
    defstruct [
      :column_name,
      :ngram,
      :feature_value,
      :feature_contribution_value
    ]
  end

  defmodule BagOfWordsCosineSimilarityFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from a bag of words cosine similarity feature group.

    ## `column_name_a`
    This is the name of the source column a for the bag of words cosine similarity feature group.

    ## `column_name_b`
    This is the name of the source column b for the bag of words cosine similarity feature group.

    ## `feature_value`
    This is the value of the feature.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type ngram :: String.t() | {String.t(), String.t()}
    @type t :: %__MODULE__{
            column_name_a: String.t(),
            column_name_b: String.t(),
            feature_value: float,
            feature_contribution_value: float
          }
    defstruct [
      :column_name_a,
      :column_name_b,
      :feature_value,
      :feature_contribution_value
    ]
  end


  defmodule WordEmbeddingFeatureContribution do
    @moduledoc """
    This describes the contribution of a feature from a word embedding feature group.

    ## `column_name`
    This is the name of the source column for the word embedding feature group.

    ## `value_index`
    This is the index of the feature in the word embedding.

    ## `feature_contribution_value`
    This is the amount that the feature contributed to the output.
    """
    @type t :: %__MODULE__{
            column_name: String.t(),
            value_index: integer,
            feature_contribution_value: float
          }
    defstruct [
      :column_name,
      :value_index,
      :feature_contribution_value
    ]
  end

  @type true_value :: String.t() | float

  defmodule LogPredictionArgs do
    @moduledoc """
    This is the type of the argument to `Tangram.log_prediction` and `Tangram.enqueue_log_prediction` which specifies the details of the prediction to log.

    ## `identifier`
    This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.

    ## `input`
    This is the same `Tangram.predict_input` value that you passed to `Tangram.predict`.

    ## `options`
    This is the same `Tangram.PredictOptions` value that you passed to `Tangram.predict`.

    ## `output`
    This is the output returned by `Tangram.predict`.
    """
    @type t :: %__MODULE__{
            identifier: String.t(),
            input: Tangram.predict_input(),
            options: PredictOptions.t() | nil,
            output: Tangram.predict_output()
          }
    defstruct [
      :identifier,
      :input,
      :options,
      :output
    ]
  end

  defmodule LogTrueValueArgs do
    @moduledoc """
    This is the type of the argument to `Tangram.log_true_value` and `Tangram.enqueue_log_true_value` which specifies the details of the true value to log.

    ## `identifier`
    This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.

    ## `true_value`
    This is the true value for the prediction.
    """
    @type t :: %__MODULE__{
            identifier: String.t(),
            true_value: Tangram.true_value()
          }
    defstruct [
      :identifier,
      :true_value
    ]
  end

  @type event :: PredictionEvent.t() | TrueValueEvent.t()

  defmodule PredictionEvent do
    @moduledoc """
    """
    @type t :: %__MODULE__{
            type: :prediction,
            model_id: String.t(),
            date: String.t(),
            identifier: String.t(),
            input: Tangram.predict_input(),
            options: PredictOptions.t() | nil,
            output: Tangram.predict_output()
          }
    @derive Jason.Encoder
    defstruct [
      :type,
      :model_id,
      :date,
      :identifier,
      :input,
      :options,
      :output
    ]
  end

  defmodule TrueValueEvent do
    @moduledoc """
    """
    @type t :: %__MODULE__{
            type: :true_value,
            model_id: String.t(),
            date: String.t(),
            identifier: String.t(),
            true_value: Tangram.true_value()
          }
    @derive Jason.Encoder
    defstruct [
      :type,
      :model_id,
      :date,
      :identifier,
      :true_value
    ]
  end

  @on_load {:init, 0}
  def init do
    sys_arch = to_string(:erlang.system_info(:system_architecture))

    nif_path =
      cond do
        String.match?(sys_arch, ~r/x86_64-(pc|unknown)-linux-gnu/) ->
          "x86_64-unknown-linux-gnu/libtangram_elixir"

        String.match?(sys_arch, ~r/(aarch64|arm)-(pc|unknown)-linux-gnu/) ->
          "aarch64-unknown-linux-gnu/libtangram_elixir"

        String.match?(sys_arch, ~r/x86_64-(alpine|pc)-linux-musl/) ->
          "x86_64-unknown-linux-musl/libtangram_elixir"

        String.match?(sys_arch, ~r/(aarch64|arm)-(alpine|pc)-linux-musl/) ->
          "aarch64-unknown-linux-musl/libtangram_elixir"

        String.match?(sys_arch, ~r/x86_64-apple-darwin[0-9]+\.[0-9]+\.[0-9]+/) ->
          "x86_64-apple-darwin/libtangram_elixir"

        String.match?(sys_arch, ~r/(aarch64|arm)-apple-darwin[0-9]+\.[0-9]+\.[0-9]+/) ->
          "aarch64-apple-darwin/libtangram_elixir"

        String.match?(sys_arch, ~r/win32/) ->
          "x86_64-pc-windows-msvc/tangram_elixir"

        true ->
          raise "Tangram for Elixir does not yet support your combination of CPU architecture and operating system. Open an issue at https://github.com/tangramdotdev/tangram/issues/new or email us at help@tangram.dev to complain."
      end

    path = :filename.join(:code.priv_dir(:tangram), nif_path)
    :ok = :erlang.load_nif(path, nil)
  end

  @doc """
  Load a model from a `.tangram` file at `path`.
  """
  @spec load_model_from_path(String.t(), LoadModelOptions | nil) :: Model.t()
  def load_model_from_path(path, options \\ nil) do
    model = _load_model_from_path(path)
    tangram_url = if options, do: options.tangram_url, else: "https://app.tangram.dev"

    %Model{
      model: model,
      log_queue: [],
      tangram_url: tangram_url
    }
  end

  @doc """
  Load a model from a binary instead of a file. You should use this only if you already have a `.tangram` loaded into memory. Otherwise, use `Tangram.load_model_from_path`, which is faster because it memory maps the file.
  """
  @spec load_model_from_binary(String.t(), LoadModelOptions | nil) :: Model.t()
  def load_model_from_binary(binary, options \\ nil) do
    model = _load_model_from_binary(binary)
    tangram_url = if options, do: options.tangram_url, else: "https://app.tangram.dev"

    %Model{
      model: model,
      log_queue: [],
      tangram_url: tangram_url
    }
  end

  @doc """
  Retrieve the model's id.
  """
  @spec model_id(Model.t()) :: String.t()
  def model_id(model) do
    _model_id(model.model)
  end

  @doc """
  Make a prediction!
  """
  @spec predict(Model.t(), Tangram.predict_input(), PredictOptions.t() | nil) ::
          Tangram.predict_output()
  def predict(model, input, options \\ nil) do
    _predict(model.model, input, options)
  end

  @doc """
  Send a prediction event to the app. If you want to batch events, you can use `Tangram.enqueue_log_prediction` instead.
  """
  @spec log_prediction(Model.t(), LogPredictionArgs.t()) :: {:ok, any} | {:error, any}
  def log_prediction(model, args) do
    event = prediction_event(model, args)
    log_events(model.tangram_url, [event])
  end

  @doc """
  Add a prediction event to the queue. Remember to call `Tangram.flush_log_queue` at a later point to send the event to the app.
  """
  @spec enqueue_log_prediction(Model.t(), LogPredictionArgs.t()) :: Model.t()
  def enqueue_log_prediction(model, args) do
    event = prediction_event(model, args)
    %{model | log_queue: model.log_queue ++ [event]}
  end

  @doc """
  Send a true value event to the app. If you want to batch events, you can use `Tangram.enqueue_log_true_value` instead.
  """
  @spec log_true_value(Model.t(), LogTrueValueArgs.t()) :: {:ok, any} | {:error, any}
  def log_true_value(model, args) do
    event = true_value_event(model, args)
    log_events(model.tangram_url, [event])
  end

  @doc """
  Add a true value event to the queue. Remember to call `Tangram.flush_log_queue` at a later point to send the event to the app.
  """
  @spec enqueue_log_true_value(Model.t(), LogTrueValueArgs.t()) :: Model.t()
  def enqueue_log_true_value(model, args) do
    event = true_value_event(model, args)
    %{model | log_queue: model.log_queue ++ [event]}
  end

  @doc """
  Send all events in the queue to the app.
  """
  @spec flush_log_queue(Model.t()) :: Model.t()
  def flush_log_queue(model) do
    log_events(model.tangram_url, model.log_queue)
    %{model | log_queue: []}
  end

  @spec log_events(String.t(), [Tangram.event()]) :: {:ok, any} | {:error, any}
  defp log_events(tangram_url, events) do
    url = tangram_url <> "/track"
    headers = %{"Content-Type": "application/json"}
    body = Jason.encode!(events)
    HTTPoison.post(url, body, headers)
  end

  @spec prediction_event(Model.t(), LogPredictionArgs.t()) :: PredictionEvent.t()
  defp prediction_event(model, args) do
    model_id = _model_id(model.model)

    %PredictionEvent{
      date: DateTime.utc_now() |> DateTime.to_iso8601(),
      identifier: args.identifier,
      input: args.input,
      model_id: model_id,
      options: args.options,
      output: args.output,
      type: :prediction
    }
  end

  @spec true_value_event(Model.t(), LogTrueValueArgs.t()) :: TrueValueEvent.t()
  defp true_value_event(model, args) do
    model_id = _model_id(model.model)

    %TrueValueEvent{
      date: DateTime.utc_now() |> DateTime.to_iso8601(),
      identifier: args.identifier,
      model_id: model_id,
      true_value: args.true_value,
      type: :true_value
    }
  end

  defp _load_model_from_path(_) do
    :erlang.nif_error(:nif_not_loaded)
  end

  defp _load_model_from_binary(_) do
    :erlang.nif_error(:nif_not_loaded)
  end

  defp _model_id(_) do
    :erlang.nif_error(:nif_not_loaded)
  end

  defp _predict(_, _, _) do
    :erlang.nif_error(:nif_not_loaded)
  end
end
