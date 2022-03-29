{
"title": "Feature Engineering"
}

ModelFox chooses the appropriate feature groups to generate based on the input column types in the CSV. ModelFox will either automatically infer the column types or you can provide the column types explicitly by passing a config file when invoking `modelfox train`. See the guide [Train with Custom Configuration](../guides/train_with_custom_configuration).

## Identity Feature Groups

_Identity_ feature groups are used for _Number_ and _Enum_ columns when training _Gradient Boosted Decision Trees_. An _Identity_ feature group for an _Enum_ column is also known as _Label Encoding_.

### Enum Column Example

For instance, suppose we have a column in our CSV called `color` which takes on three values: `Red`, `Green`, and `Blue`. The _Identity_ feature group would assign the following feature values to each of the input's enum variant values.

| Input Value | Feature Value |
| ----------- | ------------- |
| _OOV_       | 0             |
| Red         | 1             |
| Green       | 2             |
| Blue        | 3             |

The feature value `0` is reserved for _Out of Vocabulary_ (OOV) values. These are values that we did not see in the training dataset but may appear in testing or when the model is deployed.

## One Hot Encoded Feature Groups

_One Hot Encoded_ feature groups are used for _Enum_ columns when training linear models. If we were to use the _Identity_ feature encoding in linear models, we would be assuming an order: Red < Green < Blue. Therefore, we use a _One Hot Encoding_. The feature group consists of _n + 1_ features, one for each of the _Enum_ column's variants and one for the _Out of Vocabulary_ (OOV) value. The feature value at index _i_ is 1 if input value is equal to the _i_th_ variant.

### Example

For instance, suppose we have a column in our CSV called `color` which takes on three values: `Red`, `Green`, and `Blue`. The _One Hot Encoding_ feature group would assign the following feature values to each of the input's enum variant values.

| Input Value | Feature Value |
| ----------- | ------------- |
| _OOV_       | 1 0 0 0       |
| Red         | 0 1 0 0       |
| Green       | 0 0 1 0       |
| Blue        | 0 0 0 1       |

## Normalized Feature Groups

_Normalized_ feature groups are used for _Number_ columns when training linear models. The feature mapping transforms the input column values into a feature column with mean zero and unit variance.

## Bag of Words Feature Groups

_Bag of Words_ feature groups are used to encode _Text_ columns. _Bag of Words_ feature groups consist of _n_ features, one for each of the unique ngrams in the text column. The feature value for a given ngram depends on the strategy used: `Present`, `Count`, or `TF-IDF`. The `Present` strategy assigns a value of 0 or 1 depending on whether the ngram appears in the text. The `Count` strategy assigns a value equal to the count of the number of times the ngram appears in the text. The `TF-IDF` strategy assigns a value equal to the [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf) weighted count of the number of times the ngram appears in the text. See [Bag of Words Model](https://en.wikipedia.org/wiki/Bag-of-words_model) to learn more.
