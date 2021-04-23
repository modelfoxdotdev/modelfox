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
if args.library == 'pytorch' or args.library == 'sklearn':
	categorical_columns = data_train.select_dtypes(['category']).columns
	data_train.loc[:, categorical_columns] = data_train.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
	data_test.loc[:, categorical_columns] = data_test.loc[:, categorical_columns].apply(lambda x: x.cat.codes)
features_train = data_train.loc[:, data_train.columns != target_column_name]
labels_train = data_train[target_column_name]
features_test = data_test.loc[:, data_test.columns != target_column_name]
labels_test = data_test[target_column_name]

if args.library == 'pytorch' or args.library == 'sklearn':
	from sklearn.linear_model import SGDRegressor
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
	from pytorch_linear import LinearBinaryClassifier
	model = LinearBinaryClassifier(n_epochs=1, learning_rate=0.01)
	model.fit(features_train, labels_train)
elif args.library == 'sklearn':
	from sklearn.linear_model import SGDClassifier
	model = SGDClassifier(
		max_iter=1,
		eta0=0.01,
		learning_rate='constant',
		tol=None,
		loss='log'
	)
	model.fit(features_train, labels_train)

# Make predictions on the test data.
if args.library == 'pytorch':
	predictions_proba = model.predict_proba(features_test)
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
	'memory': memory
}))
