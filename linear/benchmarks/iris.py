from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.metrics import mean_squared_error
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['sklearn', 'pytorch'], required=True)
args = parser.parse_args()

# Load the data.
path_train = 'data/iris_train.csv'
path_test = 'data/iris_test.csv'
target_column_name = "species"
species_options = ['Iris Setosa', 'Iris Virginica', 'Iris Versicolor']
dtype = {
  'sepal_width': np.float64,
  'sepal_length': np.float64,
  'petal_length': np.float64,
  'petal_width': np.float64,
  'species': CategoricalDtype(categories=species_options)
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
if args.library == 'pytorch' or args.library == 'sklearn':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

if args.library == 'pytorch' or args.library == 'sklearn':
  from sklearn.preprocessing import StandardScaler
  from sklearn.compose import ColumnTransformer
  from sklearn.pipeline import Pipeline
  from sklearn.impute import SimpleImputer
  from sklearn.preprocessing import StandardScaler, OneHotEncoder
  numeric_features = features_train.select_dtypes(
    include=[np.float64, np.int64]
  ).columns
  numeric_transformer = Pipeline(steps=[
    ('scaler', StandardScaler())
  ])
  preprocessor = ColumnTransformer(
    transformers=[
      ('num', numeric_transformer, numeric_features),
  ])
  features_train = preprocessor.fit_transform(features_train)
  features_test = preprocessor.transform(features_test)

# Train the model.
if args.library == 'sklearn':
  from sklearn.linear_model import LogisticRegression
  model =  LogisticRegression(max_iter=100, multi_class='multinomial')
  model.fit(features_train, labels_train)
elif args.library == 'pytorch':
  from pytorch_linear import LinearMulticlassClassifier
  model = LinearMulticlassClassifier(n_epochs=1, learning_rate=0.1, n_classes=3)
  model.fit(features_train, labels_train)

# Make predictions on the test data.
if args.library == 'h2o':
  predictions = model.predict(data_test).as_data_frame()['predict']
else:
  predictions = model.predict(features_test)

# Compute metrics.
accuracy = accuracy_score(predictions, labels_test)

# Compute memory usage.
f = open("/proc/self/status", "r")
for line in f.readlines():
	if line.startswith("VmHWM"):
		memory = line.split(":")[1].strip()

print(json.dumps({
  'accuracy': accuracy,
  'memory': memory,
}))
