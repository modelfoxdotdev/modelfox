from pandas.api.types import CategoricalDtype
from sklearn.metrics import mean_squared_error
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost', 'catboost'], required=True)
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
if args.library == 'xgboost' or args.library == 'sklearn' or args.library == 'catboost':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
labels_train = data_train.pop(target_column_name)
features_train = data_train
labels_test = data_test.pop(target_column_name)
features_test = data_test

# Train the model.
if args.library == 'h2o':
	import h2o
	from h2o.estimators import H2OGradientBoostingEstimator
	h2o.init()
	data_train = pd.concat([features_train, labels_train], axis=1)
	data_test = pd.concat([features_test, labels_test], axis=1)
	data_train = h2o.H2OFrame(python_obj=data_train)
	data_test = h2o.H2OFrame(python_obj=data_test)
	feature_column_names = [column for column in data_train.columns if column != target_column_name]
	model = H2OGradientBoostingEstimator(
		distribution="gaussian",
		learn_rate=0.1,
		ntrees=100,
	)
	model.train(
		training_frame=data_train,
		y=target_column_name,
		x=feature_column_names,
	)
elif args.library == 'lightgbm':
	import lightgbm as lgb
	model = lgb.LGBMRegressor(
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(features_train, labels_train)
elif args.library == 'sklearn':
	from sklearn.experimental import enable_hist_gradient_boosting
	from sklearn.ensemble import HistGradientBoostingRegressor
	model = HistGradientBoostingRegressor(
		learning_rate=0.1,
		max_iter=100,
		max_leaf_nodes=255,
		validation_fraction=None,
	)
	model.fit(features_train, labels_train)
elif args.library == 'xgboost':
	import xgboost as xgb
	model = xgb.XGBRegressor(
		eta=0.1,
		eval_metric='logloss',
		grow_policy='lossguide',
		max_leaves=255,
		n_estimators=100,
		tree_method='hist',
		use_label_encoder=False,
	)
	model.fit(features_train, labels_train)
elif args.library == 'catboost':
	from catboost import CatBoostRegressor
	model = CatBoostRegressor(
		grow_policy='Lossguide',
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
		train_dir='data/catboost_info',
		verbose=False
	)
	model.fit(features_train, labels_train, silent=True)

# Make predictions on the test data.
if args.library == 'h2o':
	predictions = model.predict(data_test).as_data_frame()
else:
	predictions = model.predict(features_test)

# Compute metrics.
mse = mean_squared_error(predictions, labels_test)

print(json.dumps({
	'mse': mse,
}))
