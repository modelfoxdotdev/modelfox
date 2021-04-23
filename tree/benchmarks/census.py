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
path_train = 'data/census_train.csv'
path_test = 'data/census_test.csv'
target_column_name = "income"
workclass_options = ['State-gov', 'Self-emp-not-inc', 'Private', 'Federal-gov', 'Local-gov', '?', 'Self-emp-inc', 'Without-pay', 'Never-worked']
education_options = ['Bachelors', 'HS-grad', '11th', 'Masters', '9th', 'Some-college', 'Assoc-acdm', 'Assoc-voc', '7th-8th', 'Doctorate', 'Prof-school', '5th-6th', '10th', '1st-4th', 'Preschool', '12th']
marital_status_options = ['Never-married', 'Married-civ-spouse', 'Divorced', 'Married-spouse-absent', 'Separated', 'Married-AF-spouse', 'Widowed']
occupation_options = ['Adm-clerical', 'Exec-managerial', 'Handlers-cleaners','Prof-specialty', 'Other-service', 'Sales', 'Craft-repair', 'Transport-moving', 'Farming-fishing', 'Machine-op-inspct','Tech-support', '?', 'Protective-serv', 'Armed-Forces','Priv-house-serv']
relationship_options = ['Not-in-family', 'Husband', 'Wife', 'Own-child', 'Unmarried', 'Other-relative']
race_options = ['White', 'Black', 'Asian-Pac-Islander', 'Amer-Indian-Eskimo', 'Other']
sex_options = ['Male', 'Female']
native_country_options = ['United-States', 'Cuba', 'Jamaica', 'India', '?', 'Mexico', 'South', 'Puerto-Rico', 'Honduras', 'England', 'Canada', 'Germany', 'Iran', 'Philippines','Italy', 'Poland', 'Columbia', 'Cambodia', 'Thailand', 'Ecuador', 'Laos', 'Taiwan', 'Haiti', 'Portugal', 'Dominican-Republic', 'El-Salvador', 'France', 'Guatemala', 'China', 'Japan', 'Yugoslavia', 'Peru', 'Outlying-US(Guam-USVI-etc)', 'Scotland', 'Trinadad&Tobago', 'Greece', 'Nicaragua', 'Vietnam', 'Hong', 'Ireland', 'Hungary', 'Holand-Netherlands']
income_options = ['<=50K', '>50K']
dtype = {
	'age': np.float64,
	'workclass': CategoricalDtype(categories=workclass_options),
	'fnlwgt': np.float64,
	'education': CategoricalDtype(categories=education_options),
	'education_num': np.float64,
	'marital_status': CategoricalDtype(categories=marital_status_options),
	'occupation': CategoricalDtype(categories=occupation_options),
	'relationship': CategoricalDtype(categories=relationship_options),
	'race': CategoricalDtype(categories=race_options),
	'sex': CategoricalDtype(categories=sex_options),
	'captial_gain': np.float64,
	'captial_loss': np.float64,
	'hours_per_week': np.float64,
	'native_country': CategoricalDtype(categories=native_country_options),
	'income': CategoricalDtype(categories=income_options),
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
		learning_rate=0.1,
		n_estimators=100,
		num_leaves=255,
	)
	model.fit(
		features_train,
		labels_train
	)
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
	predictions_proba = model.predict(data_test).as_data_frame()['>50K']
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
