from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost', 'catboost'], required=True)
args = parser.parse_args()

# Load the data.
# path_train = 'data/higgs_500k_train.csv'
# path_test = 'data/higgs_500k_test.csv'
path_train = 'data/higgs_train.csv'
path_test = 'data/higgs_test.csv'
target_column_name = "signal"
dtype = {
	'signal': np.bool,
	'lepton_pt': np.float64,
	'lepton_eta': np.float64,
	'lepton_phi': np.float64,
	'missing_energy_magnitude': np.float64,
	'missing_energy_phi': np.float64,
	'jet_1_pt': np.float64,
	'jet_1_eta': np.float64,
	'jet_1_phi': np.float64,
	'jet_1_b_tag': np.float64,
	'jet_2_pt': np.float64,
	'jet_2_eta': np.float64,
	'jet_2_phi': np.float64,
	'jet_2_b_tag': np.float64,
	'jet_3_pt': np.float64,
	'jet_3_eta': np.float64,
	'jet_3_phi': np.float64,
	'jet_3_b_tag': np.float64,
	'jet_4_pt': np.float64,
	'jet_4_eta': np.float64,
	'jet_4_phi': np.float64,
	'jet_4_b_tag': np.float64,
	'm_jj': np.float64,
	'm_jjj': np.float64,
	'm_lv': np.float64,
	'm_jlv': np.float64,
	'm_bb': np.float64,
	'm_wbb': np.float64,
	'm_wwbb': np.float64,
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

# Train the model.
if args.library == 'h2o':
	import h2o
	from h2o.estimators import H2OGradientBoostingEstimator
	h2o.init(max_mem_size=20480000 * 1000)
	data_train = pd.concat([features_train, labels_train], axis=1)
	data_test = pd.concat([features_test, labels_test], axis=1)
	data_train = h2o.H2OFrame(python_obj=data_train)
	data_test = h2o.H2OFrame(python_obj=data_test)
	feature_column_names = [column for column in data_train.columns if column != target_column_name]
	model = H2OGradientBoostingEstimator(
		distribution="bernoulli",
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
		force_row_wise=True,
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
		max_leaves=255,
		eta=0.1,
		grow_policy='lossguide',
		n_estimators=100,
		tree_method='hist',
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
	predictions_proba = model.predict(data_test).as_data_frame()['True']
else:
	predictions_proba = model.predict_proba(features_test)[:, 1]

# Compute metrics.
auc_roc = roc_auc_score(labels_test, predictions_proba)

# Compute memory usage.
f = open("/proc/self/status", "r")
for line in f.readlines():
	if line.startswith("VmHWM"):
		memory = line.split(":")[1].strip()

print(json.dumps({
	'auc_roc': auc_roc,
	'memory': memory,
}))
