from pandas.api.types import CategoricalDtype
from sklearn.metrics import mean_squared_error
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['sklearn', 'pytorch'], required=True)
args = parser.parse_args()

# Load the data.
path_train = 'data/boston_train.csv'
path_test = 'data/boston_test.csv'
target_column_name = "medv"
chas_options = ["0", "1"]
dtype = {
	'crim': np.float64,
	'zn': np.float64,
	'indus': np.float64,
	'chas': CategoricalDtype(categories=chas_options),
	'nox': np.float64,
	'rm': np.float64,
	'age': np.float64,
	'dis': np.float64,
	'rad': np.int64,
	'tax': np.float64,
	'ptratio': np.float64,
	'b': np.float64,
	'lstat': np.float64,
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
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
		('imputer', SimpleImputer(strategy='median')),
		('scaler', StandardScaler())
	])
	categorical_features = features_train.select_dtypes(
		include=['category']
	).columns
	categorical_transformer = Pipeline(
		steps=[
			('imputer', SimpleImputer(strategy='constant', fill_value='missing')),
			('onehot', OneHotEncoder(handle_unknown='ignore'))
	])
	preprocessor = ColumnTransformer(
		transformers=[
			('num', numeric_transformer, numeric_features),
			('cat', categorical_transformer, categorical_features)
	])
	features_train = preprocessor.fit_transform(features_train)
	features_test = preprocessor.transform(features_test)

# Train the model.
if args.library == 'pytorch':
	from pytorch_linear import LinearRegressor
	model = LinearRegressor(n_epochs=1, learning_rate=0.01)
	model.fit(features_train, labels_train)
elif args.library == 'sklearn':
	from sklearn.linear_model import SGDRegressor
	model = SGDRegressor(
		max_iter=1,
		eta0=0.01,
		learning_rate='constant',
		tol=None,
	)
	model.fit(features_train, labels_train)

# Make predictions on the test data.
predictions = model.predict(features_test)

# Compute metrics.
mse = mean_squared_error(predictions, labels_test)

# Compute memory usage.
f = open("/proc/self/status", "r")
for line in f.readlines():
	if line.startswith("VmHWM"):
		memory = line.split(":")[1].strip()

print(json.dumps({
	'mse': mse,
	'memory': memory,
}))
