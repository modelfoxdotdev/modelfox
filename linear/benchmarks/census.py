
from pandas.api.types import CategoricalDtype
from sklearn.metrics import accuracy_score, roc_auc_score
import argparse
import numpy as np
import pandas as pd
import json

parser = argparse.ArgumentParser()
parser.add_argument('--library', choices=['sklearn', 'pytorch'], required=True)
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
	'memory': memory,
}))
