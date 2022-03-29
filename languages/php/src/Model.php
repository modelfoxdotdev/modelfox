<?php

declare(strict_types=1);

namespace modelfox\modelfox;

use RuntimeException;

final class Model
{
    /**
     * Reference to C library
     */
    private static ?\FFI $ffi = null;
    /**
     * The URL for the ModelFox app
     */
    private string $modelfox_url;
    /**
     * Log queue
     */
    private array $log_queue;
    /**
     * The ModelFox model
     */
    private \FFI\CData $model;

    /**
     * Create a new LibModelFox instance
     * @param \FFI\CData $c_model pointer to instantiated model
     * @param LoadModelOptions The options to use when loading the model
     * @return void
     */
    private function __construct(\FFI\CData $c_model, LoadModelOptions $options = null)
    {
        if (is_null(self::$ffi)) {
            // NOTE: this should never fire, both static constructors initialize it first.
            self::$ffi = Model::load_ffi();
        }
        if ($options == null || $options->modelfox_url == null) {
            $this->modelfox_url = 'https://app.modelfox.dev';
        } else {
            $this->modelfox_url = $options->modelfox_url;
        }
        $this->log_queue = [];
        $this->model = $c_model;
    }

    public function __destruct()
    {
        self::$ffi->modelfox_model_delete($this->model);
    }

    private static function load_ffi()
    {
        $cpu = php_uname('m');
        $os = php_uname('s');

        if ($os == 'Linux') {
            $ldd_output = null;
            $ldd_retval = null;
            exec('ldd --version 2>&1', $ldd_output, $ldd_retval);
            $musl = false;
            foreach ($ldd_output as $line) {
                if (strpos($line, 'musl') !== false) {
                    $musl = true;
                    break;
                }
            }
            unset($line);
            $lib = 'libmodelfox.so';
            if ($cpu == "x86_64") {
                $triple = $musl ? 'x86_64-unknown-linux-musl' : 'x86_64-unknown-linux-gnu';
            } elseif ($cpu == "aarch64") {
                $triple = $musl ? 'aarch64-unknown-linux-musl' : 'aarch64-unknown-linux-gnu';
            }
        } elseif ($cpu == 'x86_64' && $os == 'Darwin') {
            $triple = 'x86_64-apple-darwin';
            $lib = 'libmodelfox.dylib';
        } elseif (($cpu == 'arm' || $cpu == 'arm64') && $os == 'Darwin') {
            $triple = 'aarch64-apple-darwin';
            $lib = 'libmodelfox.dylib';
        } elseif ($cpu == 'x86_64' && substr($os, 0, 7) == 'Windows') {
            $triple = 'x86_64-pc-windows-msvc';
            $lib = 'modelfox.dll';
        }

        $base = '/libmodelfox/' . $triple;
        $lib_path = $base . '/' . $lib;
        $header = $base . '/modelfox.h';

        return \FFI::cdef(file_get_contents(__DIR__ . $header), __DIR__ . $lib_path);
    }

    /**
     * Load a model from the `.modelfox` file at `path`.
     *
     * @param string $path The path to the `.modelfox` file.
     * @param LoadModelOptions $options optional options to use when loading the model
     * @return Model
     */
    public static function from_path(string $path, LoadModelOptions $options = null)
    {
        if (is_null(self::$ffi)) {
            self::$ffi = Model::load_ffi();
        }
        $c_model = self::$ffi->new('modelfox_model*');
        $c_err = self::$ffi->modelfox_model_from_path($path, \FFI::addr($c_model));
        if ($c_err != null) {
            $c_error_s = new ModelFoxStringView(self::$ffi);
            self::$ffi->modelfox_error_get_message($c_err, $c_error_s->raw_ptr());
            $error_s = $c_error_s->into_string();
            self::$ffi->modelfox_error_delete($c_err);
            throw new \Exception($error_s);
        }
        $instance = new self($c_model, $options);
        return $instance;
    }

    /**
     * Load a model from bytes instead of a file.  You should use this only if you already have a `.modelfox` file loaded into memory.  Otherwise, use `Model.from_path`, which is faster because it memory maps the file.
     * @param string $bytes The bytes for the `.modelfox` model.
     * @param LoadModelOptions $options optional options to use when loading the model
     * @return Model
     */
    public static function from_bytes(string $bytes, LoadModelOptions $options = null)
    {
        if (is_null(self::$ffi)) {
            self::$ffi = Model::load_ffi();
        }
        $c_model = self::$ffi->new('modelfox_model*');
        $c_err = self::$ffi->modelfox_model_from_bytes($bytes, \FFI::addr($c_model));
        if ($c_err != null) {
            $c_error_s = new ModelFoxStringView(self::$ffi);
            self::$ffi->modelfox_error_get_message($c_err, $c_error_s->raw_ptr());
            $error_s = $c_error_s->into_string();
            self::$ffi->modelfox_error_delete($c_err);
            throw new \Exception($error_s);
        }
        $instance = new self($c_model, $options);
        return $instance;
    }

    /**
     * Make a prediction!
     * @param array $input A predict input is either a single predict input which is a map from symbols or strings to strings or floats or an array of such maps. The keys should match the columns in the CSV file you trained your model with.
     * @param PredictOptions $options Optional predict options
     * @return PredictOutput Return a single output if `input` was a single input, or an array if `input` was an array of `input`s.
     */
    public function predict(array $input, PredictOptions $options = null)
    {
        // In PHP, associative arrays are also arrays.  Instead of is_array, check if numeric key 0 is set.
        $is_array = isset($input[0]);
        // Conditionally wrap input if it isn't already an array
        $in = null;
        if ($is_array) {
            $in = $input;
        } else {
            $in = [$input];
        }

        $c_input_vec = $this->new_predict_input_vec($in);
        $c_options = $this->new_predict_options($options);
        $c_output_vec = self::$ffi->new('modelfox_predict_output_vec*');
        $c_error = self::$ffi->modelfox_model_predict($this->model, $c_input_vec, $c_options, \FFI::addr($c_output_vec));
        if ($c_error != null) {
            throw new \Exception('modelfox error');
        }
        $output = $this->predict_output_vec_from_modelfox_predict_output_vec($c_output_vec);
        self::$ffi->modelfox_predict_output_vec_delete($c_output_vec);
        if ($is_array) {
            return $output;
        } else {
            return $output[0];
        }
    }
    /**
     * Send a prediction even to the app.  If you want to batch events, you can use `enqueue_log_prediction` instead.
     * @param string $identifier This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
     * @param array $input A single PredictInput.
     * @param PredictOutput $output A single PredictOutput.
     * @param PredictOptions $options This is the same $options value that you passed to `Model::predict`.
     * @return void
     */
    public function log_prediction(string $identifier, array $input, PredictOutput $output, PredictOptions $options)
    {
        $event = $this->prediction_event($identifier, $input, $output, $options);
        $this->log_event($event);
    }

    /**
     * Add a prediction event to the queue.  Remember to call `flush_log_queue` at a later point to send the event to the app.
     * @param string $identifier This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
     * @param $input array A single prediction input.
     * @param PredictOutput $output A single prediction output
     * @param PredictOptions $options This is the same `predictOptions` value that you passed to `predict`
     * @return void
     */
    public function enqueue_log_prediction(string $identifier, array $input, PredictOutput $output, PredictOptions $options = null)
    {
        $event = $this->prediction_event($identifier, $input, $output, $options);
        array_push($this->log_queue, $event);
    }

    /**
     * Send a true value event to the app.  If you want to batch events, you can use `enqueue_log_true_value` instead.
     * @param string $identifier This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
     * @param string This is the true value for the prediction.
     * @return void
     */
    public function log_true_value(string $identifier, string $true_value)
    {
        $event = $this->true_value_event($identifier, $true_value);
        $this->log_event($event);
    }

    /**
     * Add a true value event to the queue.  Remember to call `flush_log_queue` at a later point to send the event to the app.
     * @param string $identifier This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
     * @param string $true_value This is the true value for the prediction.
     * @return void
     */
    public function enqueue_log_true_value(string $identifier, string $true_value)
    {
        $event = $this->true_value_event($identifier, $true_value);
        array_push($this->log_queue, $event);
    }

    /**
     * Send all events in the queue to the app
     * @return void
     */
    public function flush_log_queue()
    {
        $this->log_events($this->log_queue);
        $this->log_queue = [];
    }

    /**
     * Retrieve the model's ID.
     * @return string the model ID
     */
    public function id()
    {
        $c_id = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_model_get_id($this->model, $c_id->raw_ptr());
        return $c_id->into_string();
    }

    /**
     * Retrieve the currently used version of libmodelfox.
     * @return string the version number as a string
     */
    public function libmodelfox_version()
    {
        $version = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_version($version->raw_ptr());
        return $version->into_string();
    }

    private function new_predict_options(PredictOptions $options = null)
    {
        $c_options = self::$ffi->new('modelfox_predict_options*');
        self::$ffi->modelfox_predict_options_new(\FFI::addr($c_options));
        //var_dump($options);
        if ($options != null) {
            if ($options->threshold != null) {
                self::$ffi->modelfox_predict_options_set_threshold($c_options, $options->threshold);
            }
            if ($options->compute_feature_contributions != null) {
                self::$ffi->modelfox_predict_options_set_compute_feature_contributions($c_options, $options->compute_feature_contributions);
            }
        }

        return $c_options;
    }

    private function new_predict_input_vec(array $input_vec)
    {
        $c_inputs = self::$ffi->new('modelfox_predict_input_vec*');
        self::$ffi->modelfox_predict_input_vec_new(\FFI::addr($c_inputs));
        foreach ($input_vec as &$input) {
            $predict_input = $this->new_predict_input($input);
            self::$ffi->modelfox_predict_input_vec_push($c_inputs, $predict_input);
        }
        unset($input);
        return $c_inputs;
    }

    private function new_predict_input($input)
    {
        $c_input = self::$ffi->new('modelfox_predict_input*');
        self::$ffi->modelfox_predict_input_new(\FFI::addr($c_input));
        foreach ($input as $key => $value) {
            $is_float = is_float($value);
            $is_string = is_string($value);
            if ($is_float) {
                self::$ffi->modelfox_predict_input_set_value_number($c_input, $key, $value);
            } elseif ($is_string) {
                self::$ffi->modelfox_predict_input_set_value_string($c_input, $key, $value);
            } else {
                throw new \Exception('value for key' . $key . 'is not a float or a string');
            }
        }
        return $c_input;
    }

    private function predict_output_vec_from_modelfox_predict_output_vec($c_output_vec)
    {
        $outputs = [];
        $c_output_len = self::$ffi->new('size_t');
        self::$ffi->modelfox_predict_output_vec_len($c_output_vec, \FFI::addr($c_output_len));
        $len = $c_output_len->cdata;
        for ($idx = 0; $idx < $len; $idx++) {
            $c_output = self::$ffi->new('modelfox_predict_output*');
            self::$ffi->modelfox_predict_output_vec_get_at_index($c_output_vec, $idx, \FFI::addr($c_output));
            array_push($outputs, $this->predict_output_from_modelfox_predict_output($c_output));
        }

        return $outputs;
    }

    private function predict_output_from_modelfox_predict_output(\FFI\CData $c_output)
    {
        $c_task_type = self::$ffi->new('int');
        $c_task_type_variant = \FFI::cast(self::$ffi->type('modelfox_task'), \FFI::addr($c_task_type));
        self::$ffi->modelfox_model_get_task($this->model, \FFI::addr($c_task_type_variant));
        switch ($c_task_type_variant->cdata) {
            case ModelFoxTaskType::regression:
                return $this->regression_output_from_modelfox_predict_output($c_output);
            case ModelFoxTaskType::binary_classification:
                return $this->binary_classification_output_from_modelfox_predict_output($c_output);
            case ModelFoxTaskType::multiclass_classification:
                return $this->multiclass_classification_output_from_modelfox_predict_output($c_output);
        }
    }

    private function regression_output_from_modelfox_predict_output(\FFI\CData $c_output)
    {
        $c_regression_output = self::$ffi->new('modelfox_regression_predict_output*');
        self::$ffi->modelfox_predict_output_as_regression($c_output, \FFI::addr($c_regression_output));

        $c_value = self::$ffi->new('float');
        self::$ffi->modelfox_regression_predict_output_get_value($c_regression_output, \FFI::addr($c_value));
        $value = $c_value->cdata;

        $c_feature_contributions = self::$ffi->new('modelfox_feature_contributions*');
        self::$ffi->modelfox_regression_output_get_feature_contributions($c_regression_output, \FFI::addr($c_feature_contributions));
        $feature_contributions = ($c_feature_contributions == null) ? [] : $this->get_feature_contributions($c_feature_contributions);

        return new RegressionPredictOutput($value, $feature_contributions);
    }

    private function binary_classification_output_from_modelfox_predict_output(\FFI\CData $c_output)
    {
        $c_binary_classification_output = self::$ffi->new('modelfox_binary_classification_predict_output*');
        self::$ffi->modelfox_predict_output_as_binary_classification($c_output, \FFI::addr($c_binary_classification_output));

        $c_probability = self::$ffi->new('float');
        self::$ffi->modelfox_binary_classification_predict_output_get_probability($c_binary_classification_output, \FFI::addr($c_probability));
        $probability = $c_probability->cdata;

        $c_class_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_binary_classification_predict_output_get_class_name($c_binary_classification_output, $c_class_name->raw_ptr());
        $class_name = $c_class_name->into_string();

        $c_feature_contributions = self::$ffi->new('modelfox_feature_contributions*');
        self::$ffi->modelfox_binary_classification_predict_output_get_feature_contributions($c_binary_classification_output, \FFI::addr($c_feature_contributions));

        $feature_contributions = (\FFI::isNull($c_feature_contributions)) ? null : $this->get_feature_contributions($c_feature_contributions);

        return new BinaryClassificationPredictOutput($class_name, $probability, $feature_contributions);
    }

    private function multiclass_classification_output_from_modelfox_predict_output(\FFI\CData $c_output)
    {
        $c_multiclass_classification_output = self::$ffi->new('modelfox_multiclass_classification_predict_output*');
        self::$ffi->modelfox_predict_output_as_multiclass_classification($c_output, \FFI::addr($c_multiclass_classification_output));

        $c_probability = self::$ffi->new('float');
        self::$ffi->modelfox_multiclass_classification_predict_output_get_probability($c_multiclass_classification_output, \FFI::addr($c_probability));
        $probability = $c_probability->cdata;

        $c_class_name = self::$ffi->new(self::$ffi->type('modelfox_string_view'));
        self::$ffi->modelfox_multiclass_classification_predict_output_get_class_name($c_multiclass_classification_output, \FFI::addr($c_class_name));
        $class_name = substr($c_class_name->ptr, 0, $c_class_name->len);

        $probabilities = $this->multiclass_classification_output_get_probabilities($c_multiclass_classification_output);

        $feature_contributions = $this->multiclass_classification_output_get_feature_contributions($c_multiclass_classification_output);

        return new MulticlassClassificationPredictOutput($class_name, $probability, $probabilities, $feature_contributions);
    }

    private function multiclass_classification_output_get_probabilities(\FFI\CData $c_multiclass_classification_output)
    {
        $probabilities = [];
        $c_probabilities_iter = self::$ffi->new('modelfox_multiclass_classification_predict_output_probabilities_iter*');
        self::$ffi->modelfox_multiclass_classification_predict_output_get_probabilities_iter($c_multiclass_classification_output, \FFI::addr($c_probabilities_iter));

        $c_class_probability = self::$ffi->new('float');
        $c_class_name = new ModelFoxStringView(self::$ffi);
        while (!\FFI::isNull(self::$ffi->modelfox_multiclass_classification_predict_output_probabilities_iter_next(\FFI::addr($c_probabilities_iter), $c_class_name->raw_ptr(), \FFI::addr($c_class_probability)))) {
            $class_name = $c_class_name->into_string();
            $probabilities[$class_name] = $c_class_probability->cdata;
        }

        self::$ffi->modelfox_multiclass_classification_predict_output_probabilities_iter_delete($c_probabilities_iter);

        return $probabilities;
    }

    private function multiclass_classification_output_get_feature_contributions(\FFI\CData $c_multiclass_classification_output)
    {
        $c_feature_contributions_iter = self::$ffi->new('modelfox_multiclass_classification_predict_output_feature_contributions_iter*');
        self::$ffi->modelfox_multiclass_classification_predict_output_get_feature_contributions_iter($c_multiclass_classification_output, \FFI::addr($c_feature_contributions_iter));

        $feature_contributions = [];

        if (!\FFI::isNull($c_feature_contributions_iter)) {
            $c_class_name = new ModelFoxStringView(self::$ffi);
            $c_feature_contributions_ptr = self::$ffi->new('modelfox_feature_contributions*');
            while (!\FFI::isNull(self::$ffi->modelfox_multiclass_classification_predict_output_feature_contributions_iter_next(\FFI::addr($c_feature_contributions_iter), $c_class_name->raw_ptr(), \FFI::addr($c_feature_contributions_ptr)))) {
                $class_name = $c_class_name->into_string();
                $c_feature_contributions = $c_feature_contributions_ptr->cdata;
                if (!\FFI::isNull($c_feature_contributions)) {
                    $feature_contributions[$class_name] = $this->get_feature_contributions($c_feature_contributions);
                }
            }
        }

        self::$ffi->modelfox_multiclass_classification_predict_output_feature_contributions_iter_delete($c_feature_contributions_iter);

        return $feature_contributions;
    }

    private function get_feature_contributions(\FFI\CData $c_feature_contributions)
    {
        $c_baseline = self::$ffi->new('float');
        self::$ffi->modelfox_feature_contributions_get_baseline_value($c_feature_contributions, \FFI::addr($c_baseline));
        $baseline = $c_baseline->cdata;

        $c_output = self::$ffi->new('float');
        self::$ffi->modelfox_feature_contributions_get_output_value($c_feature_contributions, \FFI::addr($c_output));
        $output = $c_output->cdata;

        $feature_contribution_entries = $this->get_feature_contributions_entries($c_feature_contributions);

        return new FeatureContributions($baseline, $output, $feature_contribution_entries);
    }

    private function get_feature_contributions_entries(\FFI\CData $c_feature_contributions)
    {
        $c_len = self::$ffi->new('uint64_t');
        self::$ffi->modelfox_feature_contributions_get_entries_len($c_feature_contributions, \FFI::addr($c_len));
        $len = $c_len->cdata;

        $feature_contributions = [];
        for ($idx = 0; $idx < $len; $idx++) {
            $c_feature_contribution = self::$ffi->new('modelfox_feature_contribution_entry*');
            self::$ffi->modelfox_feature_contributions_get_entry_at_index($c_feature_contributions, $idx, \FFI::addr($c_feature_contribution));
            array_push($feature_contributions, $this->get_feature_contribution($c_feature_contribution));
        }

        return $feature_contributions;
    }

    private function get_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_feature_contribution_type = self::$ffi->new('int');
        $c_feature_contribution_type_variant = \FFI::cast(self::$ffi->type('modelfox_feature_contribution_entry_type'), \FFI::addr($c_feature_contribution_type));
        self::$ffi->modelfox_feature_contribution_entry_get_type($c_feature_contribution, \FFI::addr($c_feature_contribution_type_variant));

        switch ($c_feature_contribution_type_variant->cdata) {
            case ModelFoxFeatureContributionEntryType::identity:
                return $this->get_identity_feature_contribution($c_feature_contribution);
            case ModelFoxFeatureContributionEntryType::normalized:
                return $this->get_normalized_feature_contribution($c_feature_contribution);
            case ModelFoxFeatureContributionEntryType::one_hot_encoded:
                return $this->get_one_hot_encoded_feature_contribution($c_feature_contribution);
            case ModelFoxFeatureContributionEntryType::bag_of_words:
                return $this->get_bag_of_words_feature_contribution($c_feature_contribution);
            case ModelFoxFeatureContributionEntryType::bag_of_words_cosine_similarity:
                return $this->get_bag_of_words_cosine_similarity_feature_contribution($c_feature_contribution);
            case ModelFoxFeatureContributionEntryType::word_embedding:
                return $this->get_word_embedding_feature_contribution($c_feature_contribution);
        }
    }

    private function get_identity_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_identity_feature_contribution = self::$ffi->new('modelfox_identity_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_identity($c_feature_contribution, \FFI::addr($c_identity_feature_contribution));

        $c_column_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_identity_feature_contribution_get_column_name($c_identity_feature_contribution, $c_column_name->raw_ptr());
        $column_name = $c_column_name->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_identity_feature_contribution_get_feature_contribution_value($c_identity_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_feature_value = self::$ffi->new('float');
        self::$ffi->modelfox_identity_feature_contribution_get_feature_value($c_identity_feature_contribution, \FFI::addr($c_feature_value));
        $feature_value = $c_feature_value->cdata;

        return new IdentityFeatureContribution($column_name, $feature_value, $feature_contribution_value);
    }

    private function get_normalized_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_normalized_feature_contribution = self::$ffi->new('modelfox_normalized_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_normalized($c_feature_contribution, \FFI::addr($c_normalized_feature_contribution));

        $c_column_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_normalized_feature_contribution_get_column_name($c_normalized_feature_contribution, $c_column_name->raw_ptr());
        $column_name = $c_column_name->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_normalized_feature_contribution_get_feature_contribution_value($c_normalized_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_feature_value = self::$ffi->new('float');
        self::$ffi->modelfox_normalized_feature_contribution_get_feature_value($c_normalized_feature_contribution, \FFI::addr($c_feature_value));
        $feature_value = $c_feature_value->cdata;

        return new NormalizedFeatureContribution($column_name, $feature_value, $feature_contribution_value);
    }

    private function get_one_hot_encoded_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_one_hot_encoded_feature_contribution = self::$ffi->new('modelfox_one_hot_encoded_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_one_hot_encoded($c_feature_contribution, \FFI::addr($c_one_hot_encoded_feature_contribution));

        $c_column_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_one_hot_encoded_feature_contribution_get_column_name($c_one_hot_encoded_feature_contribution, $c_column_name->raw_ptr());
        $column_name = $c_column_name->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_one_hot_encoded_feature_contribution_get_feature_contribution_value($c_one_hot_encoded_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_feature_value = self::$ffi->new('bool');
        self::$ffi->modelfox_one_hot_encoded_feature_contribution_get_feature_value($c_one_hot_encoded_feature_contribution, \FFI::addr($c_feature_value));
        $feature_value = $c_feature_value->cdata;

        $c_variant = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_one_hot_encoded_feature_contribution_get_variant($c_one_hot_encoded_feature_contribution, $c_variant->raw_ptr());
        $variant = ($c_variant->ptr == null) ? null : $c_variant->into_string();

        return new OneHotEncodedFeatureContribution($column_name, $variant, $feature_value, $feature_contribution_value);
    }

    private function get_bag_of_words_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_bag_of_words_feature_contribution = self::$ffi->new('modelfox_bag_of_words_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_bag_of_words($c_feature_contribution, \FFI::addr($c_bag_of_words_feature_contribution));

        $c_column_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_bag_of_words_feature_contribution_get_column_name($c_bag_of_words_feature_contribution, $c_column_name->raw_ptr());
        $column_name = $c_column_name->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_bag_of_words_feature_contribution_get_feature_contribution_value($c_bag_of_words_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_feature_value = self::$ffi->new('float');
        self::$ffi->modelfox_bag_of_words_feature_contribution_get_feature_value($c_bag_of_words_feature_contribution, \FFI::addr($c_feature_value));
        $feature_value = $c_feature_value->cdata;

        $c_ngram = self::$ffi->new('modelfox_ngram*');
        $ngram = $this->get_ngram($c_ngram);

        return new BagOfWordsFeatureContribution($column_name, $ngram, $feature_contribution_value, $feature_value);
    }

    private function get_bag_of_words_cosine_similarity_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_bag_of_words_cosine_similarity_feature_contribution = self::$ffi->new('modelfox_bag_of_words_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_bag_of_words_cosine_similarity($c_feature_contribution, \FFI::addr($c_bag_of_words_cosine_similarity_feature_contribution));

        $c_column_name_a = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_bag_of_words_cosine_similarity_feature_contribution_get_column_name_a($c_bag_of_words_cosine_similarity_feature_contribution, $c_column_name_a->raw_ptr());
        $column_name_a = $c_column_name_a->into_string();

        $c_column_name_b = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_bag_of_words_cosine_similarity_feature_contribution_get_column_name_b($c_bag_of_words_cosine_similarity_feature_contribution, $c_column_name_b->raw_ptr());
        $column_name_b = $c_column_name_b->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_bag_of_words_cosine_similarity_feature_contribution_get_feature_contribution_value($c_bag_of_words_cosine_similarity_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_feature_value = self::$ffi->new('float');
        self::$ffi->modelfox_bag_of_words_cosine_similarity_feature_contribution_get_feature_value($c_bag_of_words_cosine_similarity_feature_contribution, \FFI::addr($c_feature_value));
        $feature_value = $c_feature_value->cdata;

        return new BagOfWordsCosineSimilarityFeatureContribution($column_name_a, $column_name_b, $feature_contribution_value, $feature_value);
    }

    private function get_word_embedding_feature_contribution(\FFI\CData $c_feature_contribution)
    {
        $c_word_embedding_feature_contribution = self::$ffi->new('modelfox_word_embedding_feature_contribution*');
        self::$ffi->modelfox_feature_contribution_entry_as_word_embedding($c_feature_contribution, \FFI::addr($c_word_embedding_feature_contribution));

        $c_column_name = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_word_embedding_feature_contribution_get_column_name($c_word_embedding_feature_contribution, $c_column_name->raw_ptr());
        $column_name = $c_column_name->into_string();

        $c_feature_contribution_value = self::$ffi->new('float');
        self::$ffi->modelfox_word_embedding_feature_contribution_get_feature_contribution_value($c_word_embedding_feature_contribution, \FFI::addr($c_feature_contribution_value));
        $feature_contribution_value = $c_feature_contribution_value->cdata;

        $c_value_index = self::$ffi->new('int');
        self::$ffi->modelfox_word_embedding_feature_contribution_get_feature_value($c_word_embedding_feature_contribution, \FFI::addr($c_value_index));
        $value_index = $c_value_index->cdata;

        return new WordEmbeddingFeatureContribution($column_name, $value_index, $feature_contribution_value);
    }

    private function get_ngram(\FFI\CData $ngram)
    {
        $c_ngram_type = self::$ffi->new('int');
        $c_ngram_type_variant = \FFI::cast(self::$ffi->type('modelfox_ngram_type'), \FFI::addr($c_ngram_type));
        self::$ffi->modelfox_ngram_get_type($ngram, \FFI::addr($c_ngram_type_variant));

        switch ($c_ngram_type_variant->cdata) {
            case ModelFoxNGramType::unigram:
                return $this->get_unigram_ngram($ngram);
            case ModelFoxNGramType::bigram:
                return $this->get_bigram_ngram($ngram);
        }
    }

    private function get_unigram_ngram(\FFI\CData $ngram)
    {
        $c_token = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_unigram_get_token($ngram, $c_token->raw_ptr());
        $token = $c_token->into_string();
        return new Unigram($token);
    }

    private function get_bigram_ngram(\FFI\CData $ngram)
    {
        $c_token_a = new ModelFoxStringView(self::$ffi);
        $c_token_b = new ModelFoxStringView(self::$ffi);
        self::$ffi->modelfox_bigram_get_token_a($ngram, $c_token_a->raw_ptr());
        self::$ffi->modelfox_bigram_get_token_b($ngram, $c_token_b->raw_ptr());
        $token_a = $c_token_a->into_string();
        $token_b = $c_token_a->into_string();
        return new Bigram($token_a, $token_b);
    }

    private function log_event(array $event)
    {
        $this->log_events([$event]);
    }

    private function log_events(array $events)
    {
        $content = json_encode($events);
        $content_len = strlen($content);
        $headers = [
            'Content-Type: application/json',
            'Content-Length: ' . $content_len
        ];
        $uri = $this->modelfox_url . '/track';

        $ch = curl_init($uri);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        curl_setopt($ch, CURLOPT_AUTOREFERER, 1);
        curl_setopt($ch, CURLOPT_POST, 1);
        curl_setopt($ch, CURLOPT_POSTFIELDS, $content);
        curl_setopt($ch, CURLOPT_HEADER, $headers);
        $result = curl_exec($ch);
        curl_close($ch);

        $httpcode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        if ($httpcode != 202) {
            throw new RuntimeException(json_encode($result));
        }
    }

    private function prediction_event(string $identifier, array $input, PredictOutput $output, PredictOptions $options = null)
    {
        return [
            'date' => date(DATE_RFC3339),
            'identifier' => $identifier,
            'input' => $input,
            'model_id' => $this->id(),
            'options' => $options,
            'output' => $output,
            'type' => 'prediction'
        ];
    }

    private function true_value_event(string $identifier, string $true_value)
    {
        return [
            'date' => date(DATE_RFC3339),
            'identifier' => $identifier,
            'model_id' => $this->id(),
            'true_value' => $true_value,
            'type' => 'true_value'
        ];
    }
}
