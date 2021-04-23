from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
from sklearn.model_selection import train_test_split
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['h2o', 'lightgbm', 'sklearn', 'xgboost', 'catboost'], required=True)
args = parser.parse_args()

# Load the data.
path_train = 'data/heart_disease_train.csv'
path_test = 'data/heart_disease_test.csv'
target_column_name = "diagnosis"
gender_options = ['male', 'female']
chest_pain_options = ['typical angina', 'asymptomatic', 'non-angina pain', 'atypical angina']
fasting_blood_sugar_greater_than_120_options = [True, False]
resting_ecg_result_options = ['probable or definite left ventricular hypertrophy', 'normal', 'ST-T wave abnormality']
exercise_induced_angina_options = ['no', 'yes']
exercise_st_slope_options = ['downsloping', 'flat', 'upsloping']
fluoroscopy_vessels_colored_options = ['0', '1', '2', '3']
thallium_stress_test_options = ['fixed defect', 'normal', 'reversible defect']
diagnosis_options = ['Negative', 'Positive']
dtype = {
	'age': np.float64,
	'gender': CategoricalDtype(categories=gender_options),
	'chest_pain': CategoricalDtype(categories=chest_pain_options),
	'resting_blood_pressure': np.float64,
	'cholesterol': np.float64,
	'fasting_blood_sugar_greater_than_120': CategoricalDtype(categories=fasting_blood_sugar_greater_than_120_options),
	'resting_ecg_result': CategoricalDtype(categories=resting_ecg_result_options),
	'exercise_max_heart_rate': np.float64,
	'exercise_induced_angina': CategoricalDtype(categories=exercise_induced_angina_options),
	'exercise_st_depression': np.float64,
	'exercise_st_slope': CategoricalDtype(categories=exercise_st_slope_options),
	'fluoroscopy_vessels_colored': CategoricalDtype(categories=fluoroscopy_vessels_colored_options),
	'thallium_stress_test': CategoricalDtype(categories=thallium_stress_test_options),
	'diagnosis': CategoricalDtype(categories=diagnosis_options)
}
data_train = pd.read_csv(path_train, dtype=dtype)
data_test = pd.read_csv(path_test, dtype=dtype)
if args.library == 'xgboost' or args.library == 'sklearn' or args.library == 'catboost':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

# Train the model.
if args.library == 'h2o':
	import h2o
	from h2o.estimators import H2OGradientBoostingEstimator
	h2o.init()
	h2o.no_progress()
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
		y=target_column_name,
		x=feature_column_names,
	)
elif args.library == 'lightgbm':
	import lightgbm as lgb
	model = lgb.LGBMClassifier(
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(features_train, labels_train, )
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
		tree_method='hist',
	)
	model.fit(features_train, labels_train)
elif args.library == 'catboost':
	from catboost import CatBoostClassifier
	categorical_columns = [column for column in categorical_columns if column != target_column_name]
	model = CatBoostClassifier(
		cat_features=categorical_columns,
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
	predictions_proba = model.predict(data_test).as_data_frame()['Positive']
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
