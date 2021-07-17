{
"title": "Task and Model Types"
}

## Task Types

Tangram supports training supervised machine learning models. Tangram determines the task type based on the type of the target column. If the target column is type `Number`, Tangram will train a regression model. If the target column is type `Enum` with two unique variants, Tangram will train a binary classification model. If the target column is type `Enum` with more than two variants, Tangram will train a multiclass classification model.

## Model Types

Tangram trains a hyperparameter grid consisting of both _Linear Models_ and _Gradient Boosted Decision Trees_ (GBDT).

### Linear Models

_Linear models_ are models where the relationship between the target column and the features is modeled by a linear function. Linear models are highly interpretable, offer very fast prediction times, and the model size grows linearly with the number of features. However, linear models are limited in learning only linear relationships between the features and the target column.

### Gradient Boosted Decision Trees

_Gradient Boosted Decision Trees_ (GBDT) consist of many decision tree models where each subsequent decision tree is trained to learn the error of the previous trees. GBDT's can learn non-linear relationships between the features and the target column and are among the best perfoming models for tabular data. Tangram's GBDT implementation is written entirely in Rust and has the lowest memory footprint and fastest training times as compared with the most well known GBDT implementations. See [benchmarks](https://www.tangram.xyz/benchmarks).
