{
"title": "Task and Model Types"
}

## Task Types

ModelFox determines the task, either regression, binary classification, or multiclass classification, based on the type of the target column. If the target column is a `Number` column, the task will be regression. If the target column is an `Enum` column with two variants, the task will be binary classification. If the target column is an `Enum` column with more than two variants, the task will be multiclass classification. ModelFox will automatically infer the column types or you can provide the column types explicitly in a JSON configuration file passed tot modelfox train with the `--config` flag. See the guide [Train with Custom Configuration](../guides/train_with_custom_configuration).

## Model Types

ModelFox trains a grid of _Linear_ and _Gradient Boosted Decision Tree_ (GBDT) models for the selected task. The definition of the default grid for each task is defined here: [https://github.com/modelfoxdotdev/modelfox/blob/main/crates/core/grid.rs](https://github.com/modelfoxdotdev/modelfox/blob/main/crates/core/grid.rs). Alternatively, you can specify your own grid in a JSON configuration file passed to modelfox train with the `--config` flag. See the guide [Train with Custom Configuration](../guides/train_with_custom_configuration).

### Linear Models

_Linear models_ are models where the relationship between the target column and the features is modeled by a linear function. Linear models are highly interpretable, offer very fast prediction times, and the model size grows linearly with the number of features. However, linear models are limited in learning only linear relationships between the features and the target column.

### Gradient Boosted Decision Trees

_Gradient Boosted Decision Trees_ (GBDT) consist of many decision tree models where each subsequent decision tree is trained to learn the error of the previous trees. GBDT's can learn non-linear relationships between the features and the target column and are among the best perfoming models for tabular data. ModelFox's GBDT implementation is written entirely in Rust and has the lowest memory footprint and fastest training times as compared with the most well known GBDT implementations. See [benchmarks](https://www.modelfox.dev/benchmarks).
