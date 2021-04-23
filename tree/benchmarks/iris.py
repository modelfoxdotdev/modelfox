from sklearn.metrics import accuracy_score
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost', 'catboost'], required=True)
args = parser.parse_args()

# Load the data.
path_train = 'data/iris_train.csv'
path_test = 'data/iris_test.csv'
target_column_name = "species"
data_train = pd.read_csv(path_train)
data_test = pd.read_csv(path_test)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

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
		learn_rate=0.1,
		nbins=255,
		ntrees=100,
	)
	model.train(
		training_frame=data_train,
		x=feature_column_names,
		y=target_column_name,
	)
elif args.library == 'lightgbm':
	import lightgbm as lgb
	model = lgb.LGBMClassifier(
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(features_train, labels_train)
elif args.library == 'sklearn':
	from sklearn.experimental import enable_hist_gradient_boosting
	from sklearn.ensemble import HistGradientBoostingClassifier
	model = HistGradientBoostingClassifier(
		learning_rate=0.1,
		max_iter=100,
		max_leaf_nodes=255,
		validation_fraction=None,
	)
	model.fit(features_train, labels_train)
elif args.library == 'xgboost':
	import xgboost as xgb
	model = xgb.XGBClassifier(
		eta=0.1,
		grow_policy='lossguide',
		n_estimators=100,
		tree_method='hist'
	)
	model.fit(features_train, labels_train)
elif args.library == 'catboost':
	from catboost import CatBoostClassifier
	model = CatBoostClassifier(
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
